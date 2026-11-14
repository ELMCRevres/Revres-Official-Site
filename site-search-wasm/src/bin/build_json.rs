use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct Article {
    title: String,
    url: String,
    content: String,
}

#[derive(Serialize, Clone)]
struct NewsItem {
    title: String,
    date: String,
    url: String,
    raw_date: String,
}

fn clean_text(text: &str) -> String {
    let re_br = Regex::new(r"<br\s*/?>").unwrap();
    let re_tag = Regex::new(r"<[^>]+>").unwrap();
    let re_space = Regex::new(r"\s+").unwrap();

    let t1 = re_br.replace_all(text, " ");
    let t2 = re_tag.replace_all(&t1, "");
    let t3 = re_space.replace_all(&t2, " ");
    t3.trim().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 自動化ツールを開始します ---");

    let mut base_dir = Path::new(".");
    let mut list_path = base_dir.join("list_charactor.html");

    if !list_path.exists() {
        base_dir = Path::new("..");
        list_path = base_dir.join("list_charactor.html");
    }

    if !list_path.exists() {
        eprintln!("[エラー] list_charactor.html が見つかりませんでした。");
        return Ok(());
    }

    // ==========================================
    // 1. キャラクター情報の抽出 (articles.json)
    // ==========================================
    let list_html = fs::read_to_string(&list_path)?;

    // href が "character_" から始まる <a> タグに限定することで、Topに戻る(index.html)等を完全に除外
    let card_re = Regex::new(
        r#"(?s)<a\s+[^>]*?href=["'](character_[^"']+\.html)["'][^>]*?>.*?<div\s+class=["']charactor-introduction["']>\s*<p>(.*?)</p>.*?<div\s+class=["']character-name["']>\s*<p>(.*?)</p>"#,
    )?;
    let status_re = Regex::new(r#"(?s)<div\s+class=["']status-fill["']>\s*<p>(.*?)</p>"#)?;
    let desc_re = Regex::new(r#"(?s)<div\s+class=["']description-box["']>\s*<p>(.*?)</p>"#)?;

    let mut articles: Vec<Article> = Vec::new();

    for cap in card_re.captures_iter(&list_html) {
        let url = cap[1].trim().to_string();
        let intro = clean_text(&cap[2]);
        let name = clean_text(&cap[3]);

        let mut details = vec![name.clone(), intro];

        let char_path = base_dir.join(&url);
        if char_path.exists() {
            if let Ok(char_html) = fs::read_to_string(&char_path) {
                for s_cap in status_re.captures_iter(&char_html) {
                    let cleaned = clean_text(&s_cap[1]);
                    if !cleaned.is_empty() {
                        details.push(cleaned);
                    }
                }
                if let Some(d_cap) = desc_re.captures(&char_html) {
                    details.push(clean_text(&d_cap[1]));
                }
            }
        }

        articles.push(Article {
            title: name,
            url,
            content: details.join(" "),
        });
    }

    let json_articles = serde_json::to_string_pretty(&articles)?;
    let targets = vec![
        base_dir.join("articles.json"),
        base_dir.join("site-search-wasm/articles.json"),
    ];
    for target in targets {
        if let Some(parent) = target.parent() {
            if parent.exists() {
                fs::write(&target, &json_articles)?;
            }
        }
    }
    println!("[成功] キャラクター検索データ (articles.json) を出力しました（{} 件）", articles.len());

    // ==========================================
    // 2. お知らせHTMLの収集と整列 (news.json)
    // ==========================================
    let title_re = Regex::new(r#"(?s)<h2\s+[^>]*?class=["'][^"']*news-article-title[^"']*["'][^>]*?>(.*?)</h2>"#)?;
    let date_re = Regex::new(r#"(?s)<div\s+[^>]*?class=["'][^"']*news-article-date[^"']*["'][^>]*?>.*?更新日[：:]\s*(.*?)</div>"#)?;

    let mut news_items: Vec<NewsItem> = Vec::new();

    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("news_") && file_name.ends_with(".html") && !file_name.contains("template") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let title = match title_re.captures(&content) {
                            Some(c) => clean_text(&c[1]),
                            None => file_name.to_string(),
                        };

                        let raw_date_str = match date_re.captures(&content) {
                            Some(c) => clean_text(&c[1]),
                            None => "".to_string(),
                        };

                        let formatted_date = format_japanese_date(&raw_date_str);

                        news_items.push(NewsItem {
                            title,
                            date: formatted_date,
                            url: file_name.to_string(),
                            raw_date: raw_date_str,
                        });
                    }
                }
            }
        }
    }

    news_items.sort_by(|a, b| b.raw_date.cmp(&a.raw_date));

    let json_news = serde_json::to_string_pretty(&news_items)?;
    let news_target = base_dir.join("news.json");
    fs::write(&news_target, &json_news)?;
    println!("[成功] 最新お知らせデータ (news.json) を出力しました（{} 件）", news_items.len());

    println!("--- すべて完了しました ---");
    Ok(())
}
//cargo run --bin build-json
fn format_japanese_date(date_str: &str) -> String {
    let re = Regex::new(r"(\d{4})[/-](\d{1,2})[/-](\d{1,2})").unwrap();
    if let Some(cap) = re.captures(date_str) {
        format!("{}年 {}月{}日", &cap[1], &cap[2], &cap[3])
    } else {
        date_str.to_string()
    }
}