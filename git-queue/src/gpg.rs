use std::process::Command;

pub(crate) struct GitGpg {
    program: String,
    signkey: Option<String>,
}

impl GitGpg {
    pub fn from_config(config: &git2::Config) -> Self {
        // TODO: try to discover the key to use.
        let signkey = config.get_string("user.signingkey").ok();

        let program = match config.get_str("gpg.format") {
            Ok("opengpg") => "gpg".to_string(),
            Ok("x509") => "gpgsm".to_string(),
            // Prioritize x509 format.
            // Source:
            // https://github.com/git/git/blob/75ae10bc75336db031ee58d13c5037b929235912/gpg-interface.c#L422
            _ => config
                .get_string("gpg.x509.program")
                .or_else(|_| config.get_string("gpg.opengpg.program"))
                .or_else(|_| config.get_string("gpg.program"))
                .unwrap_or_else(|_| "gpg".to_string()),
        };

        Self { program, signkey }
    }

    pub fn sign_buffer(&self, buffer: &[u8]) {
        let key = self.signkey.as_deref().unwrap();
        let cmd = Command::new(&self.program).args([
            "--status-fdd=2",
            "-bsau",
            &self.signkey.as_deref().unwrap(),
        ]);
    }
}
