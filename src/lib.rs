use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

// Convert vec![&str] to Vec<String>
macro_rules! to_vec_strings {
    ($a: expr) => {
        $a.iter().map(|s| s.to_string()).collect::<Vec<String>>()
    };
}

#[derive(Debug)]
pub struct Dockerfile {
    instructions: Vec<Vec<Vec<String>>>,
}

impl Dockerfile {
    pub fn new() -> Self {
        Dockerfile {
            instructions: vec![],
        }
    }

    pub fn parse(filename: &Path) -> Self {
        let file = File::open(filename).unwrap();
        let lines = io::BufReader::new(file).lines();

        let mut dockerfile = Dockerfile::new();
        for raw_line in lines {
            if let Ok(line) = raw_line {
                if line == "" {
                    continue;
                }

                // We assume the dockerfile is syntactically good
                let instruction: Vec<&str> = line.split(" ").collect();
                dockerfile.append(instruction);
            }
        }
        dockerfile
    }

    pub fn append(&mut self, args: Vec<&str>) -> &mut Self {
        let instr: Vec<String> = to_vec_strings!(args);
        // Every time we see the FROM keyword we create a new stage
        if args[0] == "FROM" {
            self.instructions.push(vec![instr]);
        } else {
            match self.instructions.len() {
                0 => self.instructions.push(vec![instr]),
                _ => {
                    let i = self.instructions.len() - 1;
                    self.instructions[i].push(instr);
                }
            };
            // let i = self.instructions.len() - 1;
            // self.instructions[i].push(instr);
        }

        self
    }

    // This must be the first call
    pub fn from(&mut self, image: &str) -> &mut Self {
        self.instructions
            .push(vec![vec!["FROM".to_string(), image.to_string()]]);
        self
    }

    pub fn expose(&mut self, ports: Vec<&str>) -> &mut Self {
        let mut all = vec!["EXPOSE"];
        all.append(&mut ports.to_vec());

        let row: Vec<String> = to_vec_strings!(all);
        let i = self.instructions.len() - 1;

        self.instructions[i].push(row);
        self
    }

    pub fn arg(&mut self, kv: &str) -> &mut Self {
        let i = self.instructions.len() - 1;
        self.instructions[i].push(vec!["ARG".to_string(), kv.to_string()]);
        self
    }

    pub fn run(&mut self, args: &str) -> &mut Self {
        let i = self.instructions.len() - 1;
        self.instructions[i].push(vec!["RUN".to_string(), args.to_string()]);
        self
    }

    pub fn copy(&mut self, args: &[&str]) -> &mut Self {
        let mut all = vec!["COPY"];
        all.append(&mut args.to_vec());

        let i = self.instructions.len() - 1;
        self.instructions[i].push(to_vec_strings!(all));
        self
    }

    pub fn add(&mut self, mut args: Vec<String>) -> &mut Self {
        let mut all = vec!["ADD".to_string()];
        all.append(&mut args);

        let i = self.instructions.len() - 1;
        self.instructions[i].push(all);
        self
    }

    pub fn workdir(&mut self, s: &str) -> &mut Self {
        let i = self.instructions.len() - 1;
        self.instructions[i].push(vec!["WORKDIR".to_string(), s.to_string()]);
        self
    }

    pub fn user(&mut self, name: &str) -> &mut Self {
        let i = self.instructions.len() - 1;
        self.instructions[i].push(vec!["USER".to_string(), name.to_string()]);
        self
    }

    pub fn cmd(&mut self, args: &[&str]) -> &mut Self {
        let mut all = vec!["CMD"];
        all.append(&mut args.to_vec());

        let row: Vec<String> = to_vec_strings!(all);
        let i = self.instructions.len() - 1;
        self.instructions[i].push(row);
        self
    }

    pub fn entrypoint(&mut self, mut args: Vec<String>) -> &mut Self {
        let mut all = vec!["ENTRYPOINT".to_string()];
        all.append(&mut args);

        let i = self.instructions.len() - 1;
        self.instructions[i].push(all);
        self
    }

    pub fn comment(&mut self, comment: &str) -> &mut Self {
        let i = self.instructions.len() - 1;
        self.instructions[i].push(vec!["\n# ".to_string() + comment]);
        self
    }

    pub fn stages(&self) -> usize {
        return self.instructions.len();
    }

    pub fn stage(&self, i: usize) -> &Vec<Vec<String>> {
        return &self.instructions[i];
    }

    // TODO: add a golang style "Writer" trait to write out the output
    // pub fn synth(&self) -> String {
    //     let list: Vec<String> = self
    //         .instructions
    //         .iter()
    //         .map(|instr| instr.join(" "))
    //         .collect();
    //     list.join("\n")
    // }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_dockerfile_parse() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test-Dockerfile");

        let dockerfile = Dockerfile::parse(d.as_path());
        println!("{:?}", dockerfile.instructions);
        // This is 3 due to the first line starting with a comment.
        assert_eq!(3, dockerfile.stages());
    }

    #[test]
    fn test_dockerfile() {
        let mut df = Dockerfile::new();
        df.from("rust")
            .comment("Build image")
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
            .expose(vec!["9200", "9300/tcp"])
            .arg("APP_NAME=APP_NAME")
            .copy(&[
                "--from=0",
                "/usr/src/${APP_NAME}/target/release/${APP_NAME}",
                "/usr/local/bin/${APP_NAME}",
            ])
            .cmd(&["${APP_NAME}"]);

        assert_eq!(2, df.stages());
    }
}
