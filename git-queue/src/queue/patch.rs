use git2::{Error, ErrorCode, Tree};

pub struct Patch<'r> {
    ref_name: String,
    commit: git2::Commit<'r>,
}

impl<'r> Patch<'r> {
    pub fn from_name(
        repo: &'r git2::Repository,
        branch: &str,
        name: &str,
    ) -> Result<Option<Self>, Error> {
        let ref_name = format!("refs/patches/{}/{}", branch, name);
        match repo.find_reference(&ref_name) {
            Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
            Err(error) => return Err(error),
            Ok(_ref) => {
                let commit = _ref.peel_to_commit()?;

                Ok(Some(Self { ref_name, commit }))
            }
        }
    }

    /// Full reference name of this patch.
    pub fn ref_name(&self) -> &str {
        &self.ref_name
    }

    /// Name of this patch.
    pub fn name(&self) -> &str {
        self.ref_name.rsplit_once('/').unwrap().1
    }

    /// Git object identifier of this patch.
    pub fn id(&self) -> git2::Oid {
        self.commit.id()
    }

    /// Amend this patch.
    pub fn amend(
        &mut self,
        amend: PatchAmend<'r, '_>,
        repo: &'r git2::Repository,
    ) -> Result<git2::Oid, Error> {
        let new_oid = self.commit.amend(
            Some(&self.ref_name),
            None,
            Some(&repo.signature()?),
            None,
            amend.message,
            amend.tree,
        )?;

        self.commit = repo.find_commit(new_oid)?;

        Ok(self.id())
    }
}

#[derive(Default)]
pub struct PatchAmend<'r, 's> {
    message: Option<&'s str>,
    tree: Option<&'s Tree<'r>>,
}

impl<'r, 's> PatchAmend<'r, 's> {
    pub fn set_message(&mut self, message: &'s str) {
        self.message = Some(message);
    }

    pub fn set_tree(&mut self, tree: &'s Tree<'r>) {
        self.tree = Some(tree);
    }
}
