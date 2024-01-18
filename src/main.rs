use random_word::Lang;
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct SessionConfig {
    pub article_name: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            article_name: String::from("Cheese"),
        }
    }
}

impl Clone for SessionConfig {
    fn clone(&self) -> Self {
        Self {
            article_name: self.article_name.clone(),
        }
    }
}

struct Articles {
    title: String,
    description: String,
}

fn get_options(msg: String, options: Vec<String>) -> usize {
    loop {
        clear();
        println!("{}", msg);
        options.iter().enumerate().for_each(|(i, option)| {
            println!("{}: {}", i + 1, option);
        });
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().parse::<usize>() {
            Ok(num) => {
                if num > 0 && num <= options.len() {
                    return num;
                } else {
                    println!("Please enter a valid number - press enter to continue");
                    let _ = std::io::stdin().read_line(&mut String::new());
                    continue;
                }
            }
            Err(_) => {
                println!("Please enter a valid number - press enter to continue");
                let _ = std::io::stdin().read_line(&mut String::new());
                continue;
            }
        };
    }
}

fn main() {
    let mut article: Vec<String>;
    loop {
        let option = menu();
        match option {
            1 => article = get_previous_article(),
            2 => article = search_for_article(),
            3 => article = get_random_article(),
            4 => break,
            _ => continue,
        }
        clear();
        show_article(article);
    }
}

fn show_article(article: Vec<String>) {
    for line in article {
        println!("{}", line);
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

fn get_previous_article() -> Vec<String> {
    let article: SessionConfig = confy::load("wikireader", "SessionConfig").unwrap_or_default();
    get_article(article.article_name)
}

fn search_for_article() -> Vec<String> {
    println!("Search for article");
    let mut search_result;
    let mut input = String::new();
    loop {
        println!("Enter article name: ");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let article_name = input.trim().to_owned();
        search_result = search_for_articles(article_name.to_owned());
        if search_result.is_ok() {
            break;
        } else if search_result.is_err() {
            println!("No article found with that name - press enter to continue");
            let _ = std::io::stdin().read_line(&mut String::new());
            continue;
        }
    }
    let search_result = search_result.unwrap();
    if search_result.len() == 1 {
        return get_article(search_result[0].title.to_owned());
    }
    let temp = search_result
        .iter()
        .find(|a| a.title.to_lowercase() == input.trim().to_lowercase());
    if let Some(article) = temp {
        return get_article(article.title.clone());
    }
    let options: Vec<String> = search_result
        .iter()
        .map(|a| format!("{} - {}", a.title, a.description))
        .collect();
    let option = get_options(String::from("Pick your option"), options);

    get_article(search_result[option - 1].title.clone())
}

fn get_random_article() -> Vec<String> {
    let mut word;
    loop {
        word = random_word::gen(Lang::En);
        let search_result = check_article(word.to_owned());
        if search_result {
            break;
        }
    }

    get_article(word.to_string())
}

fn check_article(search_term: String) -> bool {
    let formatted_search_term = search_term.replace(" ", "_");
    let url = format!("https://en.wikipedia.org/wiki/{}", formatted_search_term);
    let soup = Soup::new(&reqwest::blocking::get(&url).unwrap().text().unwrap());
    let query = soup.tag("b").find_all();
    for tag in query {
        if tag
            .text()
            .contains("Wikipedia does not have an article with this exact name.")
        {
            return false;
        }
    }

    let query = soup
        .tag("div")
        .attr("class", "mw-content-ltr mw-parser-output")
        .find()
        .unwrap()
        .tag("p")
        .find()
        .unwrap()
        .text();
    if query.contains("may refer to:") {
        return false;
    }
    true
}

fn search_for_articles(article_name: String) -> Result<Vec<Articles>, usize> {
    let formatted_article_name = article_name.replace(" ", "+");
    let mut articles: Vec<Articles> = Vec::new();
    let url = format!(
        "https://en.wikipedia.org/w/index.php?search={}&title=Special:Search&profile=advanced&fulltext=1&ns0=1",
        formatted_article_name
    );
    let soup = Soup::new(&reqwest::blocking::get(&url).unwrap().text().unwrap());
    let query = soup.tag("p").attr("class", "mw-search-nonefound").find();
    if query.is_some() {
        return Err(0);
    }
    let query = soup
        .tag("ul")
        .attr("class", "mw-search-results")
        .find()
        .unwrap()
        .tag("li")
        .find_all();
    for tag in query {
        let title = tag
            .tag("div")
            .attr("class", "mw-search-result-heading")
            .find()
            .unwrap()
            .tag("a")
            .find()
            .unwrap()
            .attr_name("title")
            .find()
            .unwrap()
            .text();
        let description = tag
            .tag("div")
            .attr("class", "searchresult")
            .find()
            .unwrap()
            .text();
        let url = tag
            .tag("div")
            .attr("class", "mw-search-result-heading")
            .find()
            .unwrap()
            .tag("a")
            .find()
            .unwrap()
            .attr_name("href")
            .find()
            .unwrap()
            .text();
        if description.contains("may refer to:") {
            let temp_articles = get_disambiguation_articles(url);
            articles.extend(temp_articles);
            continue;
        }
        let article = Articles {
            title: title.to_owned(),
            description: description.to_owned(),
        };
        articles.push(article);
    }
    Ok(articles)
}

fn get_article(articles: String) -> Vec<String> {
    let _search_title = articles.replace(" ", "_");
    todo!()
}

fn get_disambiguation_articles(url: String) -> Vec<Articles> {
    let soup = Soup::new(
        &reqwest::blocking::get(format!("https://en.wikipedia.org/{}", url))
            .unwrap()
            .text()
            .unwrap(),
    );
    let query = soup
        .tag("div")
        .attr("class", "mw-content-ltr mw-parser-output")
        .find()
        .unwrap()
        .tag("ul")
        .find_all();
    let mut articles: HashMap<String, Articles> = HashMap::new();
    for list in query {
        let query = list.tag("li").find_all();
        for tag in query {
            let temp = tag.tag("a").find();
            if temp.is_none() {
                continue;
            }
            let title = temp
                .clone()
                .unwrap()
                .attr_name("title")
                .find()
                .unwrap()
                .text();
            let description = tag.text();
            let url = temp.unwrap().attr_value("href").find().unwrap().text();
            let article = Articles {
                title: title.to_owned(),
                description: description.to_owned(),
            };
            articles.insert(url, article);
        }
    }
    articles.into_iter().map(|(_, v)| v).collect()
}

fn menu() -> usize {
    get_options(
        "Welcome to WikiReader".to_string(),
        vec![
            String::from("Read previous article"),
            String::from("Search for article"),
            String::from("Get random article"),
            String::from("Exit"),
        ],
    )
}

fn clear() {
    print!("{}[2J", 27 as char);
}
