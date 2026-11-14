use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// 1. 記事1件分のデータ構造を定義
#[derive(Serialize, Deserialize, Clone)]
pub struct Article {
    pub title: String,
    pub url: String,
    pub content: String,
}

// 2. ブラウザ（JavaScript）から呼び出す検索関数
#[wasm_bindgen]
pub fn search_articles(query: &str, articles_js: JsValue) -> JsValue {
    // JSのオブジェクト配列を、Rustの型安全な Vec<Article> に変換
    let articles: Vec<Article> = match serde_wasm_bindgen::from_value(articles_js) {
        Ok(data) => data,
        Err(_) => return serde_wasm_bindgen::to_value(&Vec::<Article>::new()).unwrap(),
    };

    // 検索窓が空白文字だけなら、空のリストを返して即終了
    let clean_query = query.trim().to_lowercase();
    if clean_query.is_empty() {
        return serde_wasm_bindgen::to_value(&Vec::<Article>::new()).unwrap();
    }

    // 3. タイトルまたは本文にキーワードが含まれている記事をフィルタリング
    let matched_articles: Vec<Article> = articles
        .into_iter()
        .filter(|article| {
            let title_match = article.title.to_lowercase().contains(&clean_query);
            let content_match = article.content.to_lowercase().contains(&clean_query);
            title_match || content_match
        })
        .collect();

    // 抽出された結果を、再びJavaScriptが扱えるJsValue形式に変換して返す
    serde_wasm_bindgen::to_value(&matched_articles).unwrap()
}