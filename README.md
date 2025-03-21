# Results

- Multi-indexing gives a significant advantage over indexing multiple single properties individually, taking a ~4 hour run down to ~17 minutes.
- Indexing a single derived property consisting of a tuple is indistinguishable from (simulated) "real" multi-indexing.
- The advantage of multi-indexing increases with the size of the data set.

# Test Setup

## Treatments

1. **Single indexing:** the `Age` and `County` properties are each indexed separately.
2. **Multi-indexing:** a regular (non-derived) property `AgeGroupCounty` is computed at population load time. This simulates native support for multi-indexing.
3. **Single indexing of a derived tuple:** a single derived property having `(Age, County)` values is indexed; we also call this "multi-indexing *by* derived tuple".

The properties of people in the population are:

```rust
define_person_property!(Age, u8);
define_person_property!(County, u32);
define_derived_property!(AgeGroup, u8, [Age], |age| (age/5));
```

For "multiple single indexes", this is all there is. For "multi-indexing by derived property", we add:

```rust
define_derived_property!(AgeGroupCounty, (u8, u32), [Age, County], [], |age, county| (age/5, county) );
```

For "multi-indexing", we compute a third regular (non-derived) property computed at population load time:

```rust
define_person_property!(AgeGroupCounty, (u8, u32));
```

This property and its derived variant can be indexed like any other property.

## Data Set

Synthetic data sets for Washington DC (~670K people) and California (~39M people), each as separate experimental conditions.

```
$ wc -l dc.csv
  672392 dc.csv
  
$ head -n1 ca.csv
age,homeId,schoolId,workplaceId
```

Only the first two columns, `age` and `homeId` FIPS geographic region code, were used. The county code was extracted from the `homeId`:

```rust
let county: u32 = fips[2..5].to_string().parse().unwrap(); // 3 digits for county
```

## Test Loop

The event loop was not used at all, as we only care about the effect of indexing on querying specifically. A workload of 1 million queries was done for each treatment, except when it was inconvenient, as is tradition in science.

A slightly simplified version of the test loop:

```rust
fn main() {
    let mut context = Context::new();
  
    context.init_random(42);
    // This step takes ~12s for 32M people from CA dataset. 
    population_loader::init(&mut context);

  	let indexing_time = Instant::now();
  
  	// Note that `AgeGroupCounty` is either derived, precomputed, or not defined
    // depending on the treatment. The treatment also determines which subset of
    // these are uncommented, obviously.
    // context.index_property(AgeGroup);
    // context.index_property(County);
    // context.index_property(AgeGroupCounty);
  
    // The indexing is done lazily, so we perform a "dummy" query to force it.
    // let _dummy_query = context.query_people((AgeGroupCounty, (4, 42)));
  
    println!("Indexing time: {}s", indexing_time.elapsed().as_secs_f64());
    
    let start_time = Instant::now();

    for j in 0..1_000_000 {
        // Randomly select a person
        let infector = context.sample_person(TransmissionRng, ()).expect("failed to sample person");
        // Get the infector's (derived) age group
        let age_group = context.get_person_property(infector, AgeGroup);
        let county = context.get_person_property(infector, County);

        // Query for another person with the same age group and county. The 
        // second variant is used for non-indexed and single index cases.
        let targets = context.query_people((AgeGroupCounty, (age_group, county)));
        // let targets = context.query_people(((AgeGroup, age_group), (County, county)));

        // Sample uniformly from targets
        let infectee = targets[context.sample_range(TransmissionRng, 0..targets.len())];

        // A real model would attempt an infection, etc. We just verify their 
        // age and county match the infector's, say, with an assert.
        let target_age = context.get_person_property(infectee, Age);
        let target_county = context.get_person_property(infectee, County);
        
				assert!(/* infectee and infector data coincide */);
    }
    
    let duration = start_time.elapsed();
    println!("Time elapsed is: {}s", duration.as_secs_f64());
}

```

Loading population data takes ~12 seconds for CA.

# Washington DC - 672,392 people

## Multiple single indexes

```
cargo run --release  219.37s user 0.98s system 99% cpu 3:40.83 total
```

## Multi-indexing by derived property

```
cargo run --release  160.10s user 1.14s system 99% cpu 2:42.28 total
```

# California - 39 million people

## Multi-indexing

```
Indexing time: 3.720s
Time elapsed is: 1002.708s
cargo run --release  949.87s user 58.42s system 98% cpu 16:58.96 total
```

## Multi-indexing by derived property

```
Indexing time: 3.394s
Time elapsed is: 1015.398s
cargo run --release  943.23s user 86.40s system 99% cpu 17:11.31 total
```

## Multiple single indexes

```
Indexing time: 0s
...
iterations: 200000, elapsed time: 2811.799s, projected total time: 14058.996
cargo run --release  2835.35s user 30.52s system 99% cpu 47:48.26 total
```

Time to index is neglibible.

I was too impatient to wait for the experiment to finish, so I output intermediate results with a projection and stopped when I got bored, which took ~48 minutes. Projected time was pretty stable at ~4 hours.
