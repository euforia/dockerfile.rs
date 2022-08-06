// Convert vec![&str] to Vec<String>
macro_rules! to_vec_strings {
    ($a: expr) => {
        $a.iter().map(|s| s.to_string()).collect::<Vec<String>>()
    };
}

#[derive(Debug)]
pub struct Dockerfile {
    instructions: Vec<Vec<String>>,
}

impl Dockerfile {
    pub fn new() -> Self {
        Dockerfile {
            instructions: vec![],
        }
    }

    pub fn from(&mut self, image: &str) -> &mut Self {
        self.instructions
            .push(vec!["FROM".to_string(), image.to_string()]);
        self
    }

    pub fn arg(&mut self, kv: &str) -> &mut Self {
        self.instructions
            .push(vec!["ARG".to_string(), kv.to_string()]);
        self
    }

    pub fn run(&mut self, args: &str) -> &mut Self {
        self.instructions
            .push(vec!["RUN".to_string(), args.to_string()]);
        self
    }

    pub fn copy(&mut self, args: &[&str]) -> &mut Self {
        let mut all = vec!["COPY"];
        all.append(&mut args.to_vec());

        self.instructions.push(to_vec_strings!(all));
        self
    }

    pub fn add(&mut self, mut args: Vec<String>) -> &mut Self {
        let mut all = vec!["ADD".to_string()];
        all.append(&mut args);
        self.instructions.push(all);
        self
    }

    pub fn workdir(&mut self, s: &str) -> &mut Self {
        self.instructions
            .push(vec!["WORKDIR".to_string(), s.to_string()]);
        self
    }
    pub fn user(&mut self, name: &str) -> &mut Self {
        self.instructions
            .push(vec!["USER".to_string(), name.to_string()]);
        self
    }

    pub fn cmd(&mut self, args: &[&str]) -> &mut Self {
        let mut all = vec!["CMD"];
        all.append(&mut args.to_vec());

        self.instructions.push(to_vec_strings!(all));
        self
    }

    pub fn entrypoint(&mut self, mut args: Vec<String>) -> &mut Self {
        let mut all = vec!["ENTRYPOINT".to_string()];
        all.append(&mut args);
        self.instructions.push(all);
        self
    }

    pub fn comment(&mut self, comment: &str) -> &mut Self {
        self.instructions.push(vec!["\n# ".to_string() + comment]);
        self
    }

    // TODO: add a golang style "Writer" trait to write out the output
    pub fn synth(&self) -> String {
        let list: Vec<String> = self
            .instructions
            .iter()
            .map(|instr| instr.join(" "))
            .collect();
        list.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dockerfile() {
        let mut df = Dockerfile::new();
        df.comment("Build image")
            .from("rust")
            .arg("APP_NAME")
            .workdir("/usr/src/${APP_NAME}");

        df.comment("Download and cache deps.")
            .copy(&["Cargo.toml", "Cargo.lock", "./"])
            .run("mkdir ./src && touch ./src/lib.rs")
            .run("cargo build --release")
            .run("rm -f ./src/lib.rs");

        df.comment("Build")
            .copy(&["src", "./src"])
            .run("cargo build --release");

        df.comment("Runtime image")
            .from("debian:buster-slim")
            .arg("APP_NAME=APP_NAME")
            .copy(&[
                "--from=0",
                "/usr/src/${APP_NAME}/target/release/${APP_NAME}",
                "/usr/local/bin/${APP_NAME}",
            ])
            .cmd(&["${APP_NAME}"]);

        println!("{:?}", df.synth());
    }
}
