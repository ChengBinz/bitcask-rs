use std::{fs, path::PathBuf};

use log::error;

use crate::{
    batch::{log_record_key_with_seq, parse_log_record_key},
    data::{
        data_file::{
            get_data_file_name, DataFile, HINT_FILE_NAME, MERGE_FINISHED_FILE_NAME,
            SEQ_NO_FILE_NAME,
        },
        log_record::{decode_log_record_pos, LogRecord, LogRecordType},
    },
    db::{Engine, FILE_LOCK_NAME, NON_TRANSACTION_SEQ_NO},
    errors::{Errors, Result},
    options::{IOType, Options},
};

const MERGE_DIR_NAME: &str = "merge";
const MERGE_FIN_KEY: &[u8] = "merge.finished".as_bytes();

impl Engine {
    // merge 数据目录，处理无效数据，并生成 hint 索引文件
    pub fn merge(&self) -> Result<()> {
        // 如果正在 merge 则直接返回
        let lock = self.merging_lock.try_lock();
        if lock.is_none() {
            return Err(Errors::MergeInProgress);
        }

        let merge_path = get_merge_path(self.options.dir_path.clone());
        // 如果目录已经存在，则先删除
        if merge_path.is_dir() {
            fs::remove_dir_all(merge_path.clone()).unwrap();
        }
        // 创建 merge 数据目录
        if let Err(e) = fs::create_dir_all(merge_path.clone()) {
            error!("failed to create merge path {}", e);
            return Err(Errors::FailedToCreateDatabaseDir);
        }

        // 获取所有需要进行 merge 的数据文件
        let merge_files = self.rotate_merge_files()?;

        // 打开临时用于 merge 的 bitcask 实例
        let mut merge_db_opts = Options::default();
        merge_db_opts.dir_path = merge_path.clone();
        merge_db_opts.data_file_size = self.options.data_file_size;
        let merge_db = Engine::open(merge_db_opts)?;

        // 打开 hint 文件存储索引
        let hint_file = DataFile::new_hint_file(merge_path.clone())?;
        // 依次处理每个数据文件，重写有效的数据
        for data_file in merge_files.iter() {
            let mut offset = 0;
            loop {
                let (mut log_record, size) = match data_file.read_log_record(offset) {
                    Ok(result) => (result.record, result.size),
                    Err(e) => {
                        if e == Errors::ReadDataFileEOF {
                            break;
                        }
                        return Err(e);
                    }
                };

                // 解码拿到实际的 key
                let (real_key, _) = parse_log_record_key(log_record.key.clone());
                if let Some(index_pos) = self.index.get(real_key.clone()) {
                    // 如果文件 id 和 偏移 offset 均相等，则说明是一条有效的数据
                    if index_pos.file_id == data_file.get_file_id() && index_pos.offset == offset {
                        // 去除事务的标识
                        log_record.key =
                            log_record_key_with_seq(real_key.clone(), NON_TRANSACTION_SEQ_NO);
                        let log_record_pos = merge_db.append_log_record(&mut log_record)?;
                        // 写 hint 索引
                        hint_file.write_hint_record(real_key.clone(), log_record_pos)?;
                    }
                }
                offset += size as u64;
            }
        }

        // sync 保证持久化
        merge_db.sync()?;
        hint_file.sync()?;

        // 拿到最近未参与 merge 的文件 id
        let non_merge_file_id = merge_files.last().unwrap().get_file_id() + 1;
        let merge_fin_file = DataFile::new_merge_fin_file(merge_path.clone())?;
        let merge_fin_record = LogRecord {
            key: MERGE_FIN_KEY.to_vec(),
            value: non_merge_file_id.to_string().into_bytes(),
            rec_type: LogRecordType::NOMAL,
        };
        let enc_record = merge_fin_record.encode();
        merge_fin_file.write(&enc_record)?;
        merge_fin_file.sync()?;

        Ok(())
    }

    fn rotate_merge_files(&self) -> Result<Vec<DataFile>> {
        // 取出旧的数据文件的 id
        let mut merge_file_ids = Vec::new();
        let mut older_files = self.older_files.write();
        for fid in older_files.keys() {
            merge_file_ids.push(*fid);
        }

        // 设置一个新的活跃文件用于写入
        let mut active_file = self.active_file.write();
        // sync 数据文件保证持久性
        active_file.sync()?;
        let active_file_id = active_file.get_file_id();
        let new_active_file = DataFile::new(
            self.options.dir_path.clone(),
            active_file_id + 1,
            IOType::StandardFIO,
        )?;
        *active_file = new_active_file;

        // 加到旧的数据文件当中
        let old_file = DataFile::new(
            self.options.dir_path.clone(),
            active_file_id,
            IOType::StandardFIO,
        )?;
        older_files.insert(active_file_id, old_file);

        // 加到待 merge 的文件 id 列表中
        merge_file_ids.push(active_file_id);
        // 从小到大排序，依次 merge
        merge_file_ids.sort();

        // 打开所有需要 merge 的数据文件
        let mut merge_files = Vec::new();
        for file_id in merge_file_ids.iter() {
            let data_file =
                DataFile::new(self.options.dir_path.clone(), *file_id, IOType::StandardFIO)?;
            merge_files.push(data_file);
        }

        Ok(merge_files)
    }

    /// 从 hint 索引文件中加载索引
    pub(crate) fn load_index_from_hint_file(&self) -> Result<()> {
        let hint_file_name = self.options.dir_path.join(HINT_FILE_NAME);
        // 如果 hint 文件不存在则返回
        if !hint_file_name.is_file() {
            return Ok(());
        }

        let hint_file = DataFile::new_hint_file(self.options.dir_path.clone())?;
        let mut offset = 0;
        loop {
            let (log_record, size) = match hint_file.read_log_record(offset) {
                Ok(result) => (result.record, result.size),
                Err(e) => {
                    if e == Errors::ReadDataFileEOF {
                        break;
                    }
                    return Err(e);
                }
            };

            // 解码 value，拿到位置索引信息
            let log_record_pos = decode_log_record_pos(log_record.value);
            // 存储到内存索引中
            self.index.put(log_record.key, log_record_pos);
            offset += size as u64;
        }
        Ok(())
    }
}

// 获取临时的用于 merge 的数据目录
fn get_merge_path(dir_path: PathBuf) -> PathBuf {
    let file_name = dir_path.file_name().unwrap();
    let merge_name = std::format!("{}-{}", file_name.to_str().unwrap(), MERGE_DIR_NAME);
    let parent = dir_path.parent().unwrap();
    parent.to_path_buf().join(merge_name)
}

// 加载 merge 数据目录
pub(crate) fn load_merge_files(dir_path: PathBuf) -> Result<()> {
    let merge_path = get_merge_path(dir_path.clone());
    // 没有发生过 merge 则直接返回
    if !merge_path.is_dir() {
        return Ok(());
    }

    let dir = match fs::read_dir(merge_path.clone()) {
        Ok(dir) => dir,
        Err(e) => {
            error!("failed to read merge dir: {}", e);
            return Err(Errors::FailedToReadDatabaseDir);
        }
    };

    // 查找是否有标识 merge 完成的文件
    let mut merge_file_names = Vec::new();
    let mut merge_finished = false;
    for file in dir {
        if let Ok(entry) = file {
            let file_os_str = entry.file_name();
            let file_name = file_os_str.to_str().unwrap();

            if file_name.ends_with(MERGE_FINISHED_FILE_NAME) {
                merge_finished = true;
            }
            if file_name.ends_with(SEQ_NO_FILE_NAME) {
                continue;
            }
            if file_name.ends_with(FILE_LOCK_NAME) {
                continue;
            }
            merge_file_names.push(entry.file_name());
        }
    }

    // merge 没有完成，直接返回
    if !merge_finished {
        fs::remove_dir_all(merge_path.clone()).unwrap();
        return Ok(());
    }

    // 打开标识 merge 完成的文件，取出未参与 merge 的文件 id
    let merge_fin_file = DataFile::new_merge_fin_file(merge_path.clone())?;
    let merge_fin_record = merge_fin_file.read_log_record(0)?;
    let v = String::from_utf8(merge_fin_record.record.value).unwrap();
    let non_merge_fid = v.parse::<u32>().unwrap();

    // 将旧的文件删除
    for file_id in 0..non_merge_fid {
        let file = get_data_file_name(dir_path.clone(), file_id);
        if file.is_file() {
            fs::remove_file(file).unwrap();
        }
    }

    // 将新的数据文件移动到数据目录中
    for file_name in merge_file_names {
        let src_path = merge_path.join(file_name.clone());
        let dest_path = dir_path.join(file_name.clone());
        fs::rename(src_path, dest_path).unwrap();
    }

    // 最后删除临时 merge 的目录
    fs::remove_dir_all(merge_path.clone()).unwrap();

    Ok(())
}
