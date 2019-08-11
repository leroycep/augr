pub mod event;
pub mod timesheet;

use crate::{Meta, Patch, PatchRef, Store};
use snafu::{ResultExt, Snafu};
use std::collections::{BTreeSet, VecDeque};
use timesheet::{Error as TimesheetError, PatchedTimesheet};

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum Error<IE>
where
    IE: std::error::Error + 'static,
{
    #[snafu(display("Unable to load metadata: {}", source))]
    LoadMeta { source: IE },

    #[snafu(display("Unable to save metadata: {}", source))]
    SaveMeta { source: IE },

    #[snafu(display("Unable to save patch {} to disk: {}", patch, source))]
    SavePatch { source: IE, patch: PatchRef },

    #[snafu(display("Unable to load patch {}: {}", patch, source))]
    PatchNotFound { source: IE, patch: PatchRef },

    #[snafu(display("Patch {} already loaded", patch))]
    PatchAlreadyLoaded { patch: PatchRef },

    #[snafu(display("Parents of patch {} are not loaded", patch))]
    MissingParentPatches {
        patch: PatchRef,
        parents: Vec<PatchRef>,
    },

    #[snafu(display("Patch {} could not be applied to timesheet: {:?}", patch, conflicts))]
    PatchingTimesheet {
        conflicts: Vec<TimesheetError>,
        patch: PatchRef,
    },
}

#[derive(Debug)]
pub struct Repository<S: Store> {
    store: S,
    patches_loaded: BTreeSet<PatchRef>,
    timesheet: PatchedTimesheet,
}

impl<S> Repository<S>
where
    S: Store,
    <S as Store>::Error: 'static,
{
    pub fn from_store(store: S) -> Result<Self, Vec<Error<S::Error>>> {
        let mut repo = Self {
            store,
            patches_loaded: BTreeSet::new(),
            timesheet: PatchedTimesheet::new(),
        };
        repo.load_all_patches()?;
        Ok(repo)
    }

    pub fn save_meta(&mut self) -> Result<(), Error<S::Error>> {
        let mut meta = Meta::new();
        for p in self.patches_loaded.iter() {
            meta.add_patch(p.clone());
        }
        self.store.save_meta(&meta).context(SaveMeta {})
    }

    pub fn add_patch(&mut self, patch: Patch) -> Result<(), Error<S::Error>> {
        self.load_patch(patch.clone())?;
        self.store.add_patch(&patch).context(SavePatch {
            patch: patch.patch_ref().clone(),
        })?;
        Ok(())
    }

    pub fn load_patch(&mut self, patch: Patch) -> Result<(), Error<S::Error>> {
        // Don't apply patches twice
        if self.patches_loaded.contains(patch.patch_ref()) {
            return Err(Error::PatchAlreadyLoaded {
                patch: patch.patch_ref().clone(),
            });
        }

        // Check that all of the patches parent patches have been loaded
        let mut missing_patches = Vec::new();
        for parent_patch_ref in patch.parents() {
            if !self.patches_loaded.contains(&parent_patch_ref) {
                missing_patches.push(parent_patch_ref.clone());
            }
        }
        if missing_patches.len() > 0 {
            return Err(Error::MissingParentPatches {
                patch: patch.patch_ref().clone(),
                parents: missing_patches,
            });
        }

        // Mark patch as loaded
        self.patches_loaded.insert(patch.patch_ref().clone());

        self.timesheet
            .apply_patch(&patch)
            .map_err(|conflicts| Error::PatchingTimesheet {
                patch: patch.patch_ref().clone(),
                conflicts,
            })
    }

    pub fn timesheet(&self) -> &PatchedTimesheet {
        &self.timesheet
    }

    fn load_all_patches(&mut self) -> Result<(), Vec<Error<S::Error>>> {
        let mut errors = Vec::new();

        let mut error_on_loading: BTreeSet<PatchRef> = BTreeSet::new();

        let meta = self
            .store
            .get_meta()
            .context(LoadMeta {})
            .map_err(|e| vec![e])?;
        let mut patches_to_load: VecDeque<PatchRef> = meta.patches().cloned().collect();
        while let Some(patch_ref) = patches_to_load.pop_front() {
            let patch = match self.store.get_patch(&patch_ref) {
                Ok(p) => p,
                Err(source) => {
                    errors.push(Error::PatchNotFound {
                        source,
                        patch: patch_ref.clone(),
                    });
                    continue;
                }
            };

            match self.load_patch(patch) {
                Ok(()) => {}
                Err(Error::MissingParentPatches { parents, .. }) => {
                    for parent in parents {
                        if !error_on_loading.contains(&parent) {
                            patches_to_load.push_back(parent);
                        }
                    }
                    patches_to_load.push_back(patch_ref);
                }
                Err(Error::PatchAlreadyLoaded { .. }) => {}
                Err(patch_errors) => {
                    errors.push(patch_errors);
                    error_on_loading.insert(patch_ref);
                }
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }
}
