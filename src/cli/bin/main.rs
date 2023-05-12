use std::{collections::HashMap, fs::File, io::Read};

use firestore::FirestoreDb;
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
    let db = FirestoreDb::new("esc-api-384517").await.unwrap();
    let countries = match db
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

    let mut country_scores = HashMap::new();
    for country in countries.clone() {
        country_scores.insert(country, 0);
    }
    let autocompleter = CountryHelper { countries };

    loop {
        let country = Text::new("Country")
            .with_autocomplete(autocompleter.clone())
            .with_validator(autocompleter.clone())
            .prompt()
            .unwrap();

        let score = CustomType::<u32>::new("Score").prompt().unwrap();
        country_scores.insert(country, score);
        let ranking = score_map_to_vec(country_scores.clone());

        db.fluent()
            .update()
            .in_col(ENDRESULT_COLLECTION)
            .document_id(ENDRESULT_ID)
            .object(&EndResult {
                done: false,
                countries: ranking,
            })
            .execute::<EndResult>()
            .await
            .unwrap();
    }
}

fn score_map_to_vec(map: HashMap<String, u32>) -> Vec<String> {
    let mut ranking_vec: Vec<(&String, &u32)> = map.iter().collect();
    ranking_vec.sort_by(|a, b| b.1.cmp(a.1));

    return ranking_vec.iter().map(|x| x.0.to_string()).collect();
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
