mod population_loader;

use std::time::Instant;
use ixa::{define_derived_property, define_rng, info, random, run_with_args, Context, ContextPeopleExt, ContextRandomExt};
use crate::population_loader::{Age, AgeGroup, County};

define_rng!(TransmissionRng);

// A derived property for multi-indexing
define_derived_property!(AgeGroupCounty, (u8, u32), [Age, County], [], |age, county| (age/5, county) );

fn main() {
    let indexing_time = Instant::now();
    // run_with_args(|context, _, _| {
    //     context.add_plan(1.0, |context| {
    //         println!("The current time is {}", context.get_current_time());
    //     });
    //     Ok(())
    // })
    // .unwrap();
    let mut context = Context::new();
    context.init_random(42);
    population_loader::init(&mut context);
    // context.index_property(AgeGroup);
    // context.index_property(County);
    context.index_property(AgeGroupCounty);
    let _dummy_query = context.query_people(((AgeGroup, 4), (County, 5)) );
    println!("Indexing time: {}s", indexing_time.elapsed().as_secs_f64());
    
    let mut times: Vec<f64> = Vec::with_capacity(100);
    
    let start_time = Instant::now();

    for j in 0..1_000_000 {
        // Randomly select a person
        let infector = context.sample_person(TransmissionRng, ()).expect("failed to sample person");
        // Get the infector's (derived) age group
        let age_group = context.get_person_property(infector, AgeGroup);
        let county = context.get_person_property(infector, County);

        // Query for another person with the same age group and county.
        let targets = context.query_people((AgeGroupCounty, (age_group, county)));
        // let targets = context.query_people(((AgeGroup, age_group), (County, county)));
        assert!(!targets.is_empty());
        // Sample from targets
        let infectee = targets[context.sample_range(TransmissionRng, 0..targets.len())];

        // A real model would attempt an infection, etc. We just verify their age and county
        let target_age = context.get_person_property(infectee, Age);
        let target_county = context.get_person_property(infectee, County);
        info!(
            "infector (id: {}, age group: {:?}, county: {:?}) infected (id: {}, age: {:?}, county: {:?})",
            infector,
            age_group,
            county,
            infectee,
            target_age,
            target_county
        );
        
        if (j+1) % 10_000 == 0 {
            let so_far = start_time.elapsed().as_secs_f64();
            times.push(so_far);
            let average = so_far / times.len() as f64;
            println!("iterations: {}, elapsed time: {}s, avg/10k: {}, projected total time: {}", j+1, so_far, average, average*100.0);
        }
    }
    
    let duration = start_time.elapsed();
    println!("Time elapsed is: {}s", duration.as_secs_f64());
}
