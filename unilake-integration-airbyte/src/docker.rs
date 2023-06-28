use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;

use handlebars::Handlebars;
use log::debug;
use log::error;
use log::info;

use crate::exec_stream;
use crate::ExecResult;

pub struct Docker {}

// TODO: make sure we can either download the correct binary version or integrate the correct one
impl Docker {
    pub fn build_image(
        source_image: OsString,
        target_image: OsString,
        base_image: Option<OsString>,
    ) {
        let source_image = source_image.to_string_lossy();
        let target_image = target_image.to_string_lossy();

        info!("Pulling image {}", &source_image);
        exec_stream("docker", &vec!["pull", &source_image], |line| match line {
            Ok(l) => {
                debug!("{}", l);
                ExecResult::Continue
            }
            Err(x) => {
                error!("Failed to pull image {}", &source_image);
                error!("Error: {}", x);
                ExecResult::Break
            }
        });

        info!("Preparing image {}", &source_image);
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_file("df", "dockerfile.hbs")
            .expect("Unable to register dockerfile.hbs");
        let mut data = BTreeMap::new();

        data.insert("image", &source_image);
        let bi = base_image
            .as_ref()
            .map_or_else(|| Cow::from("builder"), |a| a.to_string_lossy());
        data.insert("base_image", &bi);

        let df = handlebars.render("df", &data).unwrap();
        fs::write("Dockerfile", &df).expect("Unable to write file");

        debug!("{}", &df);
        info!("Building image: {}", &target_image);
        let mut args = vec!["build"];
        let mut succeeded = true;
        if base_image.is_some() {
            args.push("--target runtime");
        }
        args.append(&mut vec!["-t", &target_image, "."]);
        exec_stream("docker", &args, |line| match line {
            Ok(l) => {
                debug!("{}", l);
                ExecResult::Continue
            }
            Err(_) => {
                error!("Failed to build image {}", &source_image);
                succeeded = false;
                ExecResult::Break
            }
        });

        fs::remove_file("Dockerfile").expect("Unable to remove temp Dockerfile");
        info!("Done...{}!", if succeeded { "succeeded" } else { "failed" });
    }
}
