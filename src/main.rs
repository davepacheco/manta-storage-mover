/*
 * src/main.rs: driver program for playing with dependencies
 */

use std::io;
use std::process;
use std::sync::Mutex;

#[macro_use]
extern crate slog;
extern crate slog_bunyan;
use slog::Drain;

extern crate serde;
#[cfg_attr(test, macro_use)]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

const ARG0 : &str = "storage-mover";
const EXIT_FAILURE : i32 = 1;

fn main()
{
    let log = create_log();

    let mut dummycfgsource = io::Cursor::new(r#"{
        "nMaxTransfers": 8,
        "nMaxObjectSize": 1099511627776
    }"#);

    let cfg = match parse_config(&mut dummycfgsource) {
        Err(message) => fatal(message),
        Ok(c) => c
    };

    info!(log, "config: {:?}", cfg);
}

fn fatal(message: String) -> !
{
    eprintln!("{}: {}", ARG0, message);
    process::exit(EXIT_FAILURE);
}

/*
 * Logger
 */

fn create_log() ->
    slog::Logger
{
    let log_metadata = o!(
        "name" => ARG0
    );
    let log_destination = std::io::stdout();
    let log_drain = Mutex::new(slog_bunyan::default(log_destination)).fuse();
    let log = slog::Logger::root(log_drain, log_metadata);
    return log;
}

/*
 * Configuration file
 */

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MoverConfig {
    #[serde(rename = "nMaxTransfers")]  mc_nmax_transfers : u32,
    #[serde(rename = "nMaxObjectSize")] mc_nmax_object_size: u64
}

fn parse_config(r : &mut io::Read) ->
    Result<MoverConfig, String>
{
    /*
     * Read the contents of the source into memory.  (We should cap how many
     * bytes we would read.  It would be really handy if Read::read_to_string()
     * took a maximum byte count.)
     */
    let mut buffer = String::new();
    if let Err(e) = r.read_to_string(&mut buffer) {
        return Err(format!("{}", e));
    }

    /*
     * Parse the contents using serde.  Then validate a few additional
     * semantics.
     */
    let result : Result<MoverConfig, serde_json::Error> =
        serde_json::from_str(&buffer);
    let result : Result<MoverConfig, String> = match result {
        Err(e) => Err(e.to_string()),
        Ok(mcfg) => {
            if mcfg.mc_nmax_transfers == 0 {
                Err(String::from("nMaxTransfers cannot be zero"))
            } else if mcfg.mc_nmax_object_size == 0 {
                Err(String::from("nMaxObjectSize cannot be zero"))
            } else {
                Ok(mcfg)
            }
        }
    };

    if let Err(errmsg) = result {
        return Err(format!("failed to parse config: {}", errmsg))
    }

    result
}

#[cfg(test)]
mod test {
    use std::io;

    fn parse_config_str(input : &str) ->
        Result<super::MoverConfig, String>
    {
        println!("testing config:\n{}", input);
        let mut inreader = io::Cursor::new(input);
        let result = super::parse_config(&mut inreader);
        match result {
            Ok(ref cfg) => println!("result: success: {:?}\n", cfg),
            Err(ref e)  => println!("result: error: {}\n", e)
        }
        result
    }

    #[test]
    fn test_config_invalid() {
        /* Basic JSON parsing cases, including location information. */
        assert_eq!(parse_config_str(r#""#).err().unwrap(),
            "failed to parse config: EOF while parsing a \
             value at line 1 column 0");
        assert_eq!(parse_config_str(r#"{"#).err().unwrap(),
            "failed to parse config: EOF while parsing an \
             object at line 1 column 1");

        /* Missing parameters */
        assert!(parse_config_str(&json!({
            "nMaxObjectSize": 123
        }).to_string()).err().unwrap().starts_with(
            "failed to parse config: missing field `nMaxTransfers`"));
        assert!(parse_config_str(&json!({
            "nMaxTransfers": 456
        }).to_string()).err().unwrap().starts_with(
            "failed to parse config: missing field `nMaxObjectSize`"));

        /* Bad type */
        assert!(parse_config_str(&json!({
            "nMaxObjectSize": "123",
            "nMaxTransfers": 3
        }).to_string()).err().unwrap().starts_with(
            "failed to parse config: invalid type: string \"123\", \
             expected u64"));

        /* Post-serde validation */
        assert!(parse_config_str(&json!({
            "nMaxObjectSize": 10,
            "nMaxTransfers": 0
        }).to_string()).err().unwrap().starts_with(
            "failed to parse config: nMaxTransfers cannot be zero"));

        assert!(parse_config_str(&json!({
            "nMaxTransfers": 0,
            "nMaxObjectSize": 0
        }).to_string()).err().unwrap().starts_with(
            "failed to parse config: nMaxTransfers cannot be zero"));

        assert!(parse_config_str(&json!({
            "nMaxTransfers": 10,
            "nMaxObjectSize": 0
        }).to_string()).err().unwrap().starts_with(
            "failed to parse config: nMaxObjectSize cannot be zero"));
    }

    #[test]
    fn test_config_basic() {
        assert_eq!(parse_config_str(&json!({
            "nMaxTransfers": 20,
            "nMaxObjectSize": 19
        }).to_string()), Ok(super::MoverConfig {
            mc_nmax_transfers: 20,
            mc_nmax_object_size: 19
        }));
    }
}
