extern crate serde;

use actix_files as fs;
use rusoto_dynamodb::AttributeDefinition;
use rusoto_dynamodb::CreateTableInput;
use rusoto_dynamodb::KeySchemaElement;
use rusoto_dynamodb::ProvisionedThroughput;
use uuid::Uuid;


use actix_web::http::StatusCode;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DeleteItemInput, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput,
    ScanInput,
};

use std::collections::HashMap;

use dotenv;

use serde::{Deserialize, Serialize};
use serde_dynamodb;
use serde_json;
use std::env;
use std::io;
use std::vec;

// use futures::{Future, Stream};

#[derive(Serialize, Deserialize)]
pub struct Todo {
    id: Uuid,
    task: String,
}

#[derive(Deserialize)]
struct Info {
    id: String,
}

fn get_all(database: web::Data<Database>) -> Result<HttpResponse, failure::Error> {
    let scan_input = ScanInput {
        table_name: String::from("todos"),
        ..Default::default()
    };

    let items: Vec<Todo> = database
        .dynamo_db
        .scan(scan_input)
        .sync()
        .unwrap()
        .items
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|item| serde_dynamodb::from_hashmap(item).unwrap())
        .collect();

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/x-www-form-urlencoded; charset=UTF-8")
        .body(serde_json::to_string(&items)?))
}

fn add_task(text: String, database: web::Data<Database>) -> Result<HttpResponse, failure::Error> {
    let todo = Todo {
        id: Uuid::new_v4(),
        task: text,
    };
    let put_item = PutItemInput {
        item: serde_dynamodb::to_hashmap(&todo).unwrap(),
        table_name: String::from("todos"),
        ..Default::default()
    };
    match database.dynamo_db.put_item(put_item).sync() {
        Ok(_) => Ok(HttpResponse::build(StatusCode::CREATED)
            .content_type("application/x-www-form-urlencoded; charset=UTF-8")
            .body(serde_json::to_string(&todo)?)),
        Err(error) => {
            println!("Error: {:?}", error);
            Ok(HttpResponse::new(StatusCode::BAD_REQUEST))
        }
    }
}

fn delete_individual(
    path: web::Path<(Info)>,
    database: web::Data<Database>,
) -> Result<HttpResponse, failure::Error> {
    let attribute_values: HashMap<String, AttributeValue> = [(
        String::from("id"),
        AttributeValue {
            s: Some(path.id.to_string()),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();

    let delete_item = DeleteItemInput {
        key: attribute_values,
        table_name: String::from("todos"),
        ..Default::default()
    };

    match database.dynamo_db.delete_item(delete_item).sync() {
        Ok(_) => Ok(HttpResponse::new(StatusCode::OK)),
        Err(error) => {
            println!("Error: {:?}", error);
            Ok(HttpResponse::new(StatusCode::BAD_REQUEST))
        }
    }
}

fn get_individual(
    path: web::Path<Info>,
    database: web::Data<Database>,
) -> Result<HttpResponse, failure::Error> {
    let attribute_values: HashMap<String, AttributeValue> = [(
        String::from("id"),
        AttributeValue {
            s: Some(path.id.to_string()),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();

    match database
        .dynamo_db
        .get_item(GetItemInput {
            table_name: String::from("todos"),
            key: attribute_values,
            projection_expression: Some(String::from("id, task")),
            ..Default::default()
        })
        .sync()
    {
        Ok(result) => match result.item {
            Some(data) => {
                let response_body: String = serde_dynamodb::from_hashmap(data).unwrap();
                println!("{:?}", response_body);
                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/x-www-form-urlencoded; charset=UTF-8")
                    .body(response_body))
            }
            None => Ok(HttpResponse::build(StatusCode::NOT_IMPLEMENTED)
                .content_type("application/x-www-form-urlencoded; charset=UTF-8")
                .body(String::from(""))),
        },
        Err(error) => {
            println!("Error: {:?}", error);
            Ok(HttpResponse::new(StatusCode::BAD_REQUEST))
        }
    }
}

#[allow(dead_code)]
fn create_table(
    client: &DynamoDbClient,
) -> rusoto_core::RusotoResult<(), rusoto_dynamodb::CreateTableError> {
    client
        .create_table(CreateTableInput {
            attribute_definitions: vec![AttributeDefinition {
                attribute_name: "id".to_string(),
                attribute_type: "S".to_string(),
            }],
            provisioned_throughput: Some(ProvisionedThroughput {
                read_capacity_units: 5,
                write_capacity_units: 5,
            }),
            key_schema: vec![KeySchemaElement {
                attribute_name: "id".to_string(),
                key_type: "HASH".to_string(), // Partition Key
            }],
            table_name: "todos".to_string(),
            ..CreateTableInput::default()
        })
        .sync()?;
    Ok(())
}

#[allow(dead_code)]
pub struct Database {
    aws_region: Region,
    table_name: String,
    pub dynamo_db: DynamoDbClient,
}

fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let sys = actix_rt::System::new("actix-crud-v03");

    HttpServer::new(|| {
        App::new()
            // .route("/api", http::Method::GET, get_all)
            .data(Database {
                aws_region: Region::Custom {
                    name: env::var("DATABASE_NAME").unwrap(),
                    endpoint: env::var("DATABASE_URL").unwrap(),
                },
                table_name: "todos".to_string(),
                dynamo_db: DynamoDbClient::new(Region::Custom {
                    name: env::var("DATABASE_NAME").unwrap(),
                    endpoint: env::var("DATABASE_URL").unwrap(),
                }),
            })
            .service(
                web::resource("/api/tasks")
                    .route(web::get().to(get_all))
                    .route(web::post().to(add_task)),
            )
            .service(
                web::resource("/api/tasks/{id}")
                    .route(web::get().to(get_individual))
                    .route(web::delete().to(delete_individual)),
            )
            .service(fs::Files::new("/", "./client/build/").index_file("index.html"))
    })
    .bind(&env::var("BIND_ADDRESS").unwrap())?
    .start();

    println!("Server running on: {}", &env::var("BIND_ADDRESS").unwrap());

    match create_table(&DynamoDbClient::new(Region::Custom {
        name: env::var("DATABASE_NAME").unwrap(),
        endpoint: env::var("DATABASE_URL").unwrap(),
    })) {
        Ok(_) => println!("Todos table created"),
        Err(error) => panic!(error),
    }

    sys.run()
}
