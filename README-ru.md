# resq-plugin-sdk

SDK для плагинов [RESQ-kit](https://github.com/unite-q3-team/RESQ-kit).

Плагины RESQ-kit — **отдельные процессы**, которые общаются по
JSON-RPC 2.0 (newline-delimited) поверх stdio — та же транспортная модель,
что у LSP и MCP. У Rust нет стабильного ABI для плагинов, поэтому загрузка
`.dll`/`.so` внутрь Rust-гуишки хрупка; процессная граница даёт изоляцию
сбоев, версионирование протокола и возможность писать плагины на любом
языке.

Первый плагин на этом SDK: [resq-mcp](https://github.com/unite-q3-team/resq-mcp)
— MCP-сервер с инструментами анализа QVM для ИИ-агентов.

## Состав

| Модуль        | Назначение                                                       |
|---------------|------------------------------------------------------------------|
| `manifest`    | дескриптор плагина `resq-plugin.toml`                             |
| `rpc`         | конверты JSON-RPC 2.0 (`Message`, `RpcError`, коды ошибок)        |
| `transport`   | построчный фрейминг поверх любых reader/writer                    |
| `service`     | серверный цикл (`serve`, `serve_stdio`) + трейт `Handler`         |
| `handshake`   | опциональный RESQ-`initialize` для не-MCP плагинов                |

```toml
[dependencies]
resq-plugin-sdk = { path = "../resq-plugin-sdk" }
```

## Быстрый старт

```rust,ignore
use resq_plugin_sdk::{serve_stdio, Handler, PluginInfo, RpcError};
use serde_json::Value;

struct MyPlugin;

impl Handler for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo { name: "my-plugin".into(), version: "0.1.0".into() }
    }
    fn call(&mut self, method: &str, _params: &Value) -> Result<Value, RpcError> {
        Err(RpcError::method_not_found(method))
    }
}

fn main() {
    let mut p = MyPlugin;
    let _ = serve_stdio(&mut p); // читает stdin, пишет stdout; логи -> stderr
}
```

Запускаемый минимальный пример — `examples/echo.rs`.

## Правила транспорта

- **stdout — только протокол.** Вся диагностика идёт в stderr.
- Одно сообщение JSON-RPC на строку, UTF-8, завершение `\n`.
- Запросы несут `id` (число или строка); на уведомления не отвечают.
- Неизвестный метод — `-32601`; доменные сбои плагина — `-32000`
  с человекочитаемым сообщением.

## Документация

- [docs-ru/architecture.md](docs-ru/architecture.md) — почему вне-процессно, жизненный цикл, встраивание
- [docs-ru/protocol.md](docs-ru/protocol.md) — формат, коды ошибок, хендшейк
- [docs-ru/writing-a-plugin.md](docs-ru/writing-a-plugin.md) — пошаговое руководство

English version: [docs/](docs/) and [README.md](README.md).

## Лицензия

MIT — см. [LICENSE](LICENSE).
