import argv
import gleam/io
import gleam/list
import gleam/string
import httpee/commands/list as list_command
import httpee/template.{type Template}
import simplifile
import tom

pub fn main() {
  let args = argv.load().arguments
  io.debug(args)

  case args {
    ["list"] -> {
      case find_templates() {
        Error(err) -> {
          io.debug(err)
          Nil
        }
        Ok(templates) -> list_command.render(templates)
      }
    }
    _ -> {
      todo
    }
  }
}

pub fn find_templates() -> Result(List(Template), tom.GetError) {
  let assert Ok(files) = simplifile.read_directory("./example")
  let request_files =
    list.filter(files, fn(file) { string.ends_with(file, "toml") })
  let templates =
    list.map(request_files, fn(file_name) {
      let assert Ok(file_contents) = simplifile.read("./example/" <> file_name)
      let assert Ok(t) = tom.parse(file_contents)
      template.decode(t, file_name)
    })

  list.try_fold(templates, [], fn(acc, v) {
    case v {
      Error(e) -> Error(e)
      Ok(v) -> Ok(list.append(acc, [v]))
    }
  })
}
