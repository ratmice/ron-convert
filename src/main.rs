use serde_transcode;
use std::{io, io::Read as _};
use structopt::StructOpt;

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Json(serde_json::error::Error),
    Ron(ron::error::Error),
    UnsupportedFormat,
    // For stubbing things out during implementation
}

impl From<ron::error::Error> for Error {
    fn from(ron: ron::error::Error) -> Error {
        Error::Ron(ron)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(json: serde_json::error::Error) -> Error {
        Error::Json(json)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

// Should probably add the rest of the ron::ser::PrettyConfig
#[derive(Debug, StructOpt, Clone)]
struct CmdLineOptions {
    #[structopt(short, long)]
    depth_limit: Option<usize>,
    #[structopt(short, long, default_value = "RON")]
    input_format: String,
    #[structopt(short, long, default_value = "JSON")]
    output_format: String,
}

// This sure seemed like a dead end due to a buffet of weirdness
// involving differing trait bounds,
// the inability to infer the erased type (when it comes time to pass it to
// serde_transcode, and of course ownership weirdness
// Still its worth keeping in history if I feel like revisiting...
/*
enum WrapDeserializer<'a, T: 'a> {
  Ron(ron::de::Deserializer<'a>),
  Json(serde_json::de::Deserializer<T>),
}

impl<'a, T: serde_json::de::Read<'a>> WrapDeserializer<'a, T>  {
  fn from_json(json: serde_json::de::Deserializer<T>) -> Self
      where T: serde_json::de::Read<'a> + 'a
  {
    WrapDeserializer::Json(json)
  }

  fn from_ron(ron: ron::de::Deserializer<'a>) -> Self {
    WrapDeserializer::Ron(ron)
  }

  fn to_erased(&'a mut self) -> Box<dyn erased_serde::Deserializer<'a> + 'a> {
      match &mut *self {
         Self::Ron(it) => Box::new(erased_serde::Deserializer::erase(it)),
         Self::Json(it) => Box::new(erased_serde::Deserializer::erase(it)),
      }
  }
}
*/

impl CmdLineOptions {
    fn to_ron_pretty_config(&self) -> ron::ser::PrettyConfig {
        let pretty_config = ron::ser::PrettyConfig::new();
        let pretty_config = if let Some(depth_limit) = self.depth_limit {
            pretty_config.with_depth_limit(depth_limit)
        } else {
            pretty_config
        };

        pretty_config
    }
}

fn main() -> Result<(), Error> {
    let cmdline = CmdLineOptions::from_args();

    let ron_config = Some(cmdline.clone().to_ron_pretty_config());

    let struct_names = false;
    let mut input = io::stdin();
    let output = io::stdout();

    // Could surely be macro'ized into some binomial worth doing that if we add any more serialization
    // formats
    match (
        cmdline.input_format.to_uppercase().as_ref(),
        cmdline.output_format.to_uppercase().as_ref(),
    ) {
        ("JSON", "RON") => {
            let mut in_de = serde_json::de::Deserializer::from_reader(input);
            let mut out_se = ron::ser::Serializer::new(output, ron_config, struct_names)?;
            let _ = serde_transcode::transcode(&mut in_de, &mut out_se)?;
        }
        ("RON", "JSON") => {
            let mut data = String::new();
            input.read_to_string(&mut data)?;
            let mut in_de = ron::de::Deserializer::from_bytes(data.as_bytes())?;
            let mut out_se = serde_json::ser::Serializer::new(output);
            let _ = serde_transcode::transcode(&mut in_de, &mut out_se)?;
        }
        ("RON", "RON") => {
            let mut data = String::new();
            input.read_to_string(&mut data)?;
            let mut in_de = ron::de::Deserializer::from_bytes(data.as_bytes())?;
            let mut out_se = ron::ser::Serializer::new(output, ron_config, struct_names)?;
            let _ = serde_transcode::transcode(&mut in_de, &mut out_se)?;
        }
        ("JSON", "JSON") => {
            let mut in_de = serde_json::de::Deserializer::from_reader(input);
            let mut out_se = serde_json::ser::Serializer::new(output);
            let _ = serde_transcode::transcode(&mut in_de, &mut out_se)?;
        }
        (_, _) => return Err(Error::UnsupportedFormat),
    }
    Ok(())
}
