use csv::ReaderBuilder;
use std::error::Error;
use ixa::{define_derived_property, define_person_property, define_rng, info, warn, Context, ContextRandomExt};
use ixa::people::ContextPeopleExt;

const AGE_DISTRIBUTION_PATH: &str = "../CDC/data/USAgeDistribution.csv";
const POPULATION_PATH: &str = "../CDC/data/ca.csv";

define_rng!(SyntheticPopRng);

define_person_property!(Age, u8);
define_person_property!(County, u32);
// define_person_property!(AgeGroupCounty, (u8, u32));
define_derived_property!(AgeGroup, u8, [Age], |age| (age/5));

#[derive(Debug)]
struct FipsCode {
  state: u32,
  county: u32,
  tract: u32,
  block: u32,
}

// Function that parses a FIPS code and returns a FipsCode struct
fn parse_fips_code(fips: &str) -> Result<FipsCode, &'static str> {
  // Ensure the FIPS code is exactly 15 characters long
  if fips.len() != 15 {
    return Err("Invalid FIPS code length. It must be 15 digits.");
  }

  // Extract the components
  let state: u32 = fips[0..2].to_string().parse().unwrap();  // First 2 digits for state
  let county: u32 = fips[2..5].to_string().parse().unwrap(); // Next 3 digits for county
  let tract: u32 = fips[5..11].to_string().parse().unwrap(); // Next 6 digits for tract
  let block: u32 = fips[11..15].to_string().parse().unwrap(); // Last 4 digits for block

  // Return the struct with the extracted values
  Ok(FipsCode {
    state,
    county,
    tract,
    block,
  })
}

fn get_age_distribution() -> Result<Vec<u32>, Box<dyn Error>> {
  let mut csv_reader = ReaderBuilder::new()
      .has_headers(true)
      .from_path(AGE_DISTRIBUTION_PATH)?;

  let mut counts: Vec<u32> = Vec::new();

  for result in csv_reader.records() {
    let record = result?;
    if let Some(count_str) = record.get(1) { // The 'Count' column is at index 1
      let count: u32 = count_str.parse()?;
      counts.push(count);
    }
  }

  // Print out the counts (or further process them)
  info!("Read the age distribution {:?}", counts);

  Ok(counts)
}

pub fn init(context: &mut Context) {
  // let weights = get_age_distribution().expect("failed to read age distribution");

  // Read in the population data. We only use the first column for homeid.
  let mut csv_reader = ReaderBuilder::new()
      .has_headers(true)
      .from_path(POPULATION_PATH)
      .expect("failed to read population data");

  for result in csv_reader.records() {
    if let Ok(record) = result{
      if let (Some(age), Some(home_id)) = (record.get(0), record.get(1) ){ // age and homeId are column 0 and 1
        if let (Ok(age), Ok(fips_code)) = (age.parse::<u8>(), parse_fips_code(&home_id)) {

          // context.add_person(((Age, age), (County, fips_code.county), (AgeGroupCounty, (age/5, fips_code.county))))
          context.add_person(((Age, age), (County, fips_code.county)))
                 .expect("failed to add person");
          
        } else { 
          warn!("Could not parse fips code {:?}", home_id);
        }
      } else { 
        warn!("No home ID found: {:?}", record);
      }
    }
  }
}
