use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Clone, Debug)]
enum SubCommand {
    GetAllProjects,
    GetProject { project_id: String },
    UpdateItem { item_id: String, done: String },
    NewList { project_id: String, name: String },
    NewListItem { list_id: String, name: String },
    DeleteList { list_id: String },
    DeleteListItem { item_id: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct Id {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    id: String,
    name: String,
    color: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,

    lists: Option<Vec<Id>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct List {
    id: String,
    name: String,
    #[serde(rename = "projectId")]
    project_id: String,
    items: Vec<ListItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListItem {
    id: String,
    name: String,
    done: bool,
    #[serde(rename = "listId")]
    list_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FullProject {
    id: String,
    name: String,
    color: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,

    lists: Vec<List>,
}

async fn get_all_projects() -> Option<Vec<Project>> {
    let result = reqwest::get("http://localhost:3000/api/project/all")
        .await
        .ok()?
        .json::<Vec<Project>>()
        .await
        .ok()?;
    Some(result)
}

async fn get_project(id: &str) -> Option<Project> {
    // TODO(patrik): Handle errors
    let url = format!("http://localhost:3000/api/project?id={}", id);
    let result = reqwest::get(url).await.ok()?.json::<Project>().await.ok()?;
    Some(result)
}

async fn get_project_list(list_id: &str) -> Option<List> {
    let url = format!("http://localhost:3000/api/project/list?id={}", list_id);
    let result = reqwest::get(url).await.ok()?.json::<List>().await.ok()?;
    Some(result)
}

async fn get_full_project(project_id: &str) -> Option<FullProject> {
    let project = get_project(project_id).await?;
    let mut lists = Vec::new();
    for list in project.lists.as_ref()?.iter() {
        let list = get_project_list(&list.id).await?;
        lists.push(list);
    }

    Some(FullProject {
        id: project.id,
        name: project.name,
        color: project.color,
        created_at: project.created_at,
        updated_at: project.updated_at,

        lists,
    })
}

async fn update_item(item_id: &str, done: bool) -> bool {
    let url = "http://localhost:3000/api/project/list/item";
    let client = reqwest::Client::new();
    let value = json!({
        "id": item_id,
        "data": {
            "done": done,
        }
    });
    let res = client.patch(url).json(&value).send().await;

    if let Ok(res) = res {
        res.status().is_success()
    } else {
        false
    }
}

async fn new_list(project_id: &str, name: &str) -> Option<String> {
    let url = "http://localhost:3000/api/project/list";
    let client = reqwest::Client::new();
    let data = json!({
        "name": name,
        "projectId": project_id,
    });
    let res = client.post(url).json(&data).send().await;

    if let Ok(res) = res {
        if res.status().is_success() {
            return Some(res.text().await.unwrap());
        }

        None
    } else {
        None
    }
}

async fn new_list_item(list_id: &str, name: &str) -> Option<String> {
    let url = "http://localhost:3000/api/project/list/item";
    let client = reqwest::Client::new();
    let data = json!({
        "name": name,
        "listId": list_id,
    });
    let res = client.post(url).json(&data).send().await;

    if let Ok(res) = res {
        if res.status().is_success() {
            return Some(res.text().await.unwrap());
        }

        None
    } else {
        None
    }
}

async fn delete_list(list_id: &str) -> bool {
    // TODO(patrik): Url encode the list id
    let url = format!("http://localhost:3000/api/project/list?id={}", list_id);
    let client = reqwest::Client::new();
    let res = client.delete(url).send().await;

    if let Ok(res) = res {
        res.status().is_success()
    } else {
        false
    }
}

async fn delete_list_item(item_id: &str) -> bool {
    // TODO(patrik): Url encode the list id
    let url = format!("http://localhost:3000/api/project/list/item?id={}", item_id);
    let client = reqwest::Client::new();
    let res = client.delete(url).send().await;

    if let Ok(res) = res {
        res.status().is_success()
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        SubCommand::GetAllProjects => {
            let projects = get_all_projects().await.unwrap();
            print!("{}", serde_json::to_string_pretty(&projects).unwrap())
        }

        SubCommand::GetProject { project_id } => {
            let project = get_full_project(&project_id).await.unwrap();
            print!("{}", serde_json::to_string_pretty(&project).unwrap());
        }

        SubCommand::UpdateItem { item_id, done } => {
            if !update_item(&item_id, done == "true").await {
                panic!("Failed to update item");
            }
        }

        SubCommand::NewList { project_id, name } => {
            if let Some(res) = new_list(&project_id, &name).await {
                println!("{}", res)
            } else {
                panic!("Failed to create list");
            }
        }

        SubCommand::NewListItem { list_id, name } => {
            if let Some(res) = new_list_item(&list_id, &name).await {
                println!("{}", res)
            } else {
                panic!("Failed to create list item");
            }
        }

        SubCommand::DeleteList { list_id } => {
            if !delete_list(&list_id).await {
                panic!("Failed to delete list");
            }
        }

        SubCommand::DeleteListItem { item_id } => {
            if !delete_list_item(&item_id).await {
                panic!("Failed to delete item");
            }
        }
    }

    Ok(())
}
