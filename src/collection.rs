use hdf5::File;
use ptree::{item::StringItem, TreeBuilder};
use std::path::{Path, PathBuf};

/// An error arising from loading a collection
#[derive(Debug, thiserror::Error)]
#[error("Failed to read collection")]
#[allow(clippy::missing_docs_in_private_items)]
pub enum Error {
    #[error("Error encountered when reading from Hdf5: {0}")]
    Hdf5(#[from] hdf5::Error),
    #[error("Could not determine stem of NeXus file")]
    NoFileStem,
    #[error("Could not determine parent direcotry of NeXus file")]
    NoParentDirecory,
    #[error("Dataset {0} not found in file")]
    DatasetNotFound(String),
}

/// A detector module
#[derive(Debug)]
pub struct Module {
    /// The data files written by the module
    data_files: Vec<File>,
}

/// A data collection
#[derive(Debug)]
pub struct Collection {
    /// A set of detector modules
    modules: Vec<Module>,
}

impl Collection {
    /// Load a [`Collection`] from the NeXus file definition
    pub fn from_nexus(path: PathBuf, datafile_zero_padding: usize) -> Result<Self, Error> {
        let file = File::open(&path)?;
        let meta = file.group("/entry/data/meta_file")?;

        let module_file_counts = meta.dataset("fp_per_module")?.read_1d::<u32>()?;
        let datafile_prefix = path
            .file_stem()
            .ok_or(Error::NoFileStem)?
            .to_str()
            .ok_or(Error::NoFileStem)?;
        let directory = path.parent().ok_or(Error::NoParentDirecory)?.to_owned();

        let mut modules = Vec::new();
        let mut file_number_offset = 0;
        for module_file_count in module_file_counts {
            let mut data_files = Vec::new();
            for file_idx in 1..=module_file_count {
                let file_number = file_number_offset + file_idx;
                let data_file_name =
                    format!("{datafile_prefix}_{file_number:0>datafile_zero_padding$}.h5");
                let mut data_file_path = directory.clone();
                data_file_path.push(data_file_name);
                data_files.push(File::open(&data_file_path)?);
            }
            modules.push(Module { data_files });
            file_number_offset += module_file_count;
        }

        Ok(Self { modules })
    }

    /// Produces a [`ptree`] tree for degug visualisation
    pub fn as_tree(&self) -> StringItem {
        let mut tree = TreeBuilder::new("collection".to_string());
        for (module_idx, module) in self.modules.iter().enumerate() {
            tree.begin_child(format!("module_{module_idx}"));
            for data_file in module.data_files.iter() {
                tree.add_empty_child(data_file.filename());
            }
            tree.end_child();
        }
        tree.build()
    }
}

/// Event
pub struct Event {
    /// List of Event Id
    event_id: Vec<u32>,
    /// X and Y position of an event
    position: Vec<(u16, u16)>,
}

impl Event {
    /// Read a event
    pub fn read_event<P: AsRef<Path>>(path: P) -> Result<Vec<u32>, Error> {
        let file = File::open(path)?;
        if let Ok(dataset) = file.dataset("event_id") {
            let values: Vec<u32> = dataset.read_1d()?.to_vec();
            Ok(values)
        } else {
            Err(Error::DatasetNotFound("event_id".to_string()))
        }
    }
}

/// Reads HDF5 file
pub fn read_hdf5_data<P: AsRef<Path>>(path: P, keys: &[&str]) -> Result<Vec<Vec<f64>>, Error> {
    let file = File::open(path)?;
    let mut data = Vec::new();

    for &key in keys {
        if let Ok(dataset) = file.dataset(key) {
            let values: Vec<f64> = dataset.read_1d()?.to_vec();
            data.push(values);
        } else {
            return Err(Error::DatasetNotFound(key.to_string()));
        }
    }

    Ok(data)
}
