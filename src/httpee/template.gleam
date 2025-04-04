import gleam/dict.{type Dict}
import gleam/list
import gleam/option.{type Option}
import gleam/result
import tom

pub type RequestDetails {
  RequestDetails(
    url: String,
    method: String,
    headers: Dict(String, String),
    body: Option(String),
  )
}

pub type Template {
  Template(
    name: String,
    file_name: String,
    description: String,
    variables: Dict(String, String),
    request_details: RequestDetails,
  )
}

pub fn decode(
  toml: Dict(String, tom.Toml),
  file_name: String,
) -> Result(Template, tom.GetError) {
  let request_detail = decode_request_details(toml)
  let variables = toml_string_dict(toml, ["variables"])

  use name <- result.try(tom.get_string(toml, ["name"]))
  use description <- result.try(tom.get_string(toml, ["description"]))
  use variables <- result.try(variables)
  use request_details <- result.try(request_detail)

  Ok(Template(file_name:, name:, description:, variables:, request_details:))
}

fn decode_request_details(
  toml: Dict(String, tom.Toml),
) -> Result(RequestDetails, tom.GetError) {
  let headers = toml_string_dict(toml, ["request", "headers"])

  use url <- result.try(tom.get_string(toml, ["request", "url"]))
  use method <- result.try(tom.get_string(toml, ["request", "method"]))
  use headers <- result.try(headers)
  let body = toml |> tom.get_string(["request", "body"]) |> option.from_result()

  Ok(RequestDetails(url:, method:, headers:, body:))
}

fn toml_string_dict(
  toml: Dict(String, tom.Toml),
  parent: List(String),
) -> Result(Dict(String, String), tom.GetError) {
  use table <- result.try(tom.get_table(toml, parent))

  table
  |> dict.map_values(fn(key, _value) {
    tom.get_string(toml, list.append(parent, [key]))
  })
  |> dict.to_list()
  |> list.try_fold(dict.new(), fn(acc, value) {
    let #(key, val) = value
    case val {
      Error(err) -> Error(err)
      Ok(v) -> Ok(dict.insert(acc, key, v))
    }
  })
}
