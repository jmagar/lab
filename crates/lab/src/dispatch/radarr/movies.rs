//! Movie resource dispatch: list, get, lookup, add, edit, delete.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use lab_apis::radarr::types::movie::TmdbId;
use lab_apis::radarr::types::{Movie, MovieId};
use serde_json::Value;

use super::params::{require_i64, require_str, to_json};
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "movie.list",
        description: "List all movies in the Radarr library",
        destructive: false,
        returns: "Movie[]",
        params: &[],
    },
    ActionSpec {
        name: "movie.get",
        description: "Get a single movie by its Radarr ID",
        destructive: false,
        returns: "Movie",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Radarr movie ID",
        }],
    },
    ActionSpec {
        name: "movie.lookup",
        description: "Search for movies to add (TMDB / IMDB lookup)",
        destructive: false,
        returns: "MovieLookup[]",
        params: &[ParamSpec {
            name: "query",
            ty: "string",
            required: true,
            description: "Search term, TMDB ID (tmdb:12345), or IMDB ID (imdb:tt1234567)",
        }],
    },
    ActionSpec {
        name: "movie.add",
        description: "Add a movie to Radarr for monitoring and download",
        destructive: false,
        returns: "Movie",
        params: &[
            ParamSpec {
                name: "tmdb_id",
                ty: "i64",
                required: true,
                description: "TMDB ID of the movie",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: true,
                description: "Movie title",
            },
            ParamSpec {
                name: "quality_profile_id",
                ty: "i64",
                required: true,
                description: "Quality profile ID (get from quality-profile.list)",
            },
            ParamSpec {
                name: "root_folder_path",
                ty: "string",
                required: true,
                description: "Root folder path (get from root-folder.list)",
            },
            ParamSpec {
                name: "monitored",
                ty: "bool",
                required: false,
                description: "Monitor movie for download (default true)",
            },
            ParamSpec {
                name: "year",
                ty: "i32",
                required: false,
                description: "Release year (default 0)",
            },
        ],
    },
    ActionSpec {
        name: "movie.edit",
        description: "Update an existing movie resource (PUT full resource)",
        destructive: false,
        returns: "Movie",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Radarr movie ID to update",
            },
            ParamSpec {
                name: "body",
                ty: "object",
                required: true,
                description: "Full movie resource JSON (fetch via movie.get, modify, send back)",
            },
        ],
    },
    ActionSpec {
        name: "movie.delete",
        description: "Delete a movie from Radarr",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Radarr movie ID",
            },
            ParamSpec {
                name: "delete_files",
                ty: "bool",
                required: false,
                description: "Also delete files from disk (default false)",
            },
        ],
    },
];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "movie.list" => {
            let movies = client.movie_list().await?;
            to_json(movies)
        }
        "movie.get" => {
            let id = MovieId(require_i64(&params, "id")?);
            let movie = client.movie_get(id).await?;
            to_json(movie)
        }
        "movie.lookup" => {
            let query = require_str(&params, "query")?;
            let results = client.movie_lookup(query).await?;
            to_json(results)
        }
        "movie.add" => {
            let tmdb_id = require_i64(&params, "tmdb_id")?;
            let title = require_str(&params, "title")?.to_owned();
            let quality_profile_id = params.get("quality_profile_id").and_then(Value::as_i64);
            let root_folder_path = params
                .get("root_folder_path")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let monitored = params
                .get("monitored")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let year = params
                .get("year")
                .and_then(Value::as_i64)
                .map_or(0, |y| i32::try_from(y).unwrap_or(0));
            let movie = Movie {
                id: MovieId(0),
                title,
                year,
                tmdb_id: TmdbId(tmdb_id),
                imdb_id: None,
                has_file: false,
                monitored,
                quality_profile_id,
                root_folder_path,
                path: None,
                size_on_disk: 0,
            };
            let added = client.movie_add(&movie).await?;
            to_json(added)
        }
        "movie.edit" => {
            let id = MovieId(require_i64(&params, "id")?);
            let body = params
                .get("body")
                .cloned()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `body`".to_string(),
                    param: "body".to_string(),
                })?;
            let updated = client.movie_edit(id, &body).await?;
            Ok(updated)
        }
        "movie.delete" => {
            let id = MovieId(require_i64(&params, "id")?);
            let delete_files = params
                .get("delete_files")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            client.movie_delete(id, delete_files).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        _ => unreachable!(),
    }
}
