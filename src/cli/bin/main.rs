use std::{fs::File, io::Read};

use firestore::{FirestoreDb, FirestoreDbOptions};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use inquire::{
    autocompletion::Replacement,
    validator::{StringValidator, Validation},
    Autocomplete, CustomType, Text,
};
use serde::{Deserialize, Serialize};

const ENDRESULT_COLLECTION: &str = "endresult";
const ENDRESULT_ID: &str = "endresult_id";

#[derive(Deserialize, Serialize, Debug, Clone)]
struct EndResult {
    done: bool,
    countries: Vec<String>,
}

#[tokio::main]
async fn main() {
    let db = FirestoreDb::with_options_service_account_key_file(
        FirestoreDbOptions::new("esc-2024-422706".to_string()),
        "esc-2024-422706-4e4c77825a5c.json".into(),
    )
    .await
    .unwrap();
    let mut ranking = match db
        .fluent()
        .select()
        .by_id_in(ENDRESULT_COLLECTION)
        .obj::<EndResult>()
        .one(ENDRESULT_ID.to_string())
        .await
        .unwrap()
    {
        Some(end_result) => end_result.countries,
        None => get_countries(),
    };

    for (i, country) in ranking.iter().enumerate() {
        println!("{}: {country}", i + 1)
    }

    let autocompleter = CountryHelper {
        countries: ranking.clone(),
    };

    loop {
        let country = Text::new("Country")
            .with_autocomplete(autocompleter.clone())
            .with_validator(autocompleter.clone())
            .prompt()
            .unwrap();

        let position = CustomType::<usize>::new("Position").prompt().unwrap() - 1;
        ranking.remove(ranking.iter().position(|x| *x == country).unwrap());
        ranking.insert(position, country);

        for (i, country) in ranking.iter().enumerate() {
            println!("{}: {country}", i + 1)
        }

        db.fluent()
            .update()
            .in_col(ENDRESULT_COLLECTION)
            .document_id(ENDRESULT_ID)
            .object(&EndResult {
                done: false,
                countries: ranking.clone(),
            })
            .execute::<EndResult>()
            .await
            .unwrap();
    }
}

fn get_countries() -> Vec<String> {
    let mut file = File::open("countries.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    serde_json::from_str::<Vec<String>>(&data).unwrap()
}

#[derive(Clone)]
struct CountryHelper {
    countries: Vec<String>,
}

impl StringValidator for CountryHelper {
    fn validate(
        &self,
        input: &str,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        return match self.countries.contains(&input.to_string()) {
            true => Ok(Validation::Valid),
            false => Ok(Validation::Invalid(
                inquire::validator::ErrorMessage::Default,
            )),
        };
    }
}

impl Autocomplete for CountryHelper {
    fn get_suggestions(&mut self, _input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        Ok(Vec::new())
    }

    fn get_completion(
        &mut self,
        input: &str,
        _highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let matcher = SkimMatcherV2::default();

        let mut high_score = 0;
        let mut suggestion = String::new();
        for country in self.countries.clone() {
            match matcher.fuzzy_match(&country, input) {
                Some(score) => {
                    if score > high_score {
                        suggestion = country;
                        high_score = score;
                    }
                }
                None => (),
            }
        }

        return Ok(match suggestion.is_empty() {
            true => Replacement::None,
            false => Replacement::Some(suggestion),
        });
    }
}
