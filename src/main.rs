use colored::Colorize;
use random_word::Lang;
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

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

impl Clone for Articles {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            description: self.description.clone(),
        }
    }
}

fn get_options(msg: String, options: Vec<String>) -> usize {
    loop {
        clear();
        println!("{}", msg.bold().red());
        options.iter().enumerate().for_each(|(i, option)| {
            println!("{}: {}", i + 1, option.blue());
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
    let mut title: String;
    loop {
        let option = menu();
        match option {
            1 => (article, title) = get_previous_article(),
            2 => (article, title) = search_for_article(),
            3 => (article, title) = get_random_article(),
            4 => break,
            _ => continue,
        }
        clear();
        show_article(article, title);
    }
}

fn show_article(article: Vec<String>, title: String) {
    println!("{}", title.blue().bold());
    for line in article {
        println!("{}", line);
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

fn get_previous_article() -> (Vec<String>, String) {
    let article: SessionConfig = confy::load("wikireader", "SessionConfig").unwrap_or_default();
    (
        get_article(article.article_name.clone()),
        article.article_name,
    )
}

fn search_for_article() -> (Vec<String>, String) {
    let mut search_result;
    let mut input = String::new();
    loop {
        clear();
        println!("{}", "Enter article name: ".bold().red());
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
        return (
            get_article(search_result[0].title.to_owned()),
            search_result[0].title.clone(),
        );
    }
    let temp = search_result
        .iter()
        .find(|a| a.title.to_lowercase() == input.trim().to_lowercase());
    if let Some(article) = temp {
        return (get_article(article.title.clone()), article.title.clone());
    }
    let options: Vec<String> = search_result
        .iter()
        .map(|a| format!("{} - {}", a.title, a.description))
        .collect();
    let option = get_options(String::from("Pick your option"), options);

    (
        get_article(search_result[option - 1].title.clone()),
        search_result[option - 1].title.clone(),
    )
}

fn get_random_article() -> (Vec<String>, String) {
    let mut word;
    loop {
        word = random_word::gen(Lang::En);
        let search_result = check_article(format!("/wiki/{}", word.to_owned()));
        if search_result {
            break;
        }
    }

    (get_article(word.to_string()), word.to_string())
}

fn check_article(search_term: String) -> bool {
    let formatted_search_term = search_term.replace(" ", "_");
    let url = format!("https://en.wikipedia.org{}", formatted_search_term);
    let soup = Soup::new(&reqwest::blocking::get(&url).unwrap().text().unwrap());
    let query = soup.tag("b").find_all();
    for tag in query {
        if tag
            .text()
            .contains("Wikipedia does not have an article with this exact name.")
        {
            return false;
        } else if tag.text().contains("Wikipedia does not have an article on") {
            return false;
        }
    }

    let query = soup
        .tag("a")
        .attr("href", "/wiki/Category:Disambiguation_pages")
        .find();
    if query.is_some() {
        return false;
    }

    true
}

fn search_for_articles(article_name: String) -> Result<Vec<Articles>, usize> {
    let formatted_article_name = article_name.replace(" ", "+");
    let mut articles: HashMap<String, Articles> = HashMap::new();
    let url = format!(
        "https://en.wikipedia.org/w/index.php?search={}&title=Special:Search&profile=advanced&fulltext=1&ns0=1",
        formatted_article_name
    );
    let response = reqwest::blocking::get(&url);
    if response.is_err() {
        return Err(0);
    }
    let soup = Soup::new(&response.unwrap().text().unwrap());
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
        let url = tag.tag("a").find().unwrap().get("href").unwrap();

        if !check_article(url.to_owned()) {
            let temp_articles = get_disambiguation_articles(url);
            articles.extend(temp_articles);
            continue;
        }
        let article = Articles {
            title: title.to_owned(),
            description: description.to_owned(),
        };
        articles.insert(url, article);
    }
    let articles = articles
        .iter()
        .map(|(_, v)| (*v).clone())
        .collect::<Vec<Articles>>();
    Ok(articles)
}

fn get_article(articles: String) -> Vec<String> {
    let search_title = articles.replace(" ", "_");
    println!("{}", articles.blue().bold());
    let url = format!("https://en.wikipedia.org/wiki/{}", search_title);
    let response = reqwest::blocking::get(&url).unwrap().text().unwrap();
    let soup = Soup::new(&response);

    // Save soup result to a temporary text file
    let temp_file_path = "/home/arman/projects/wikireader/temp_file.txt";
    let mut temp_file = File::create(temp_file_path).unwrap();
    temp_file.write_all(response.as_bytes()).unwrap();

    let query = soup
        .tag("div")
        .attr("id", "mw-content-text")
        .find()
        .unwrap()
        .tag("p")
        .find_all();
    let query = query
        .into_iter()
        .map(|tag| tag.text())
        .collect::<Vec<String>>();
    query
}

fn get_disambiguation_articles(url: String) -> HashMap<String, Articles> {
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
            let url = temp.unwrap().get("href").unwrap();
            let article = Articles {
                title: title.to_owned(),
                description: description.to_owned(),
            };
            articles.insert(url, article);
        }
    }
    articles
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
