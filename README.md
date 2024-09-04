
# Simple WKD

A simple web key directory server written in rust.


![AGPLv3 License](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)


## Features

- Simple web interface to manage keys
- Darkmode support
- Email confirmations
- Support for both the `Advanced` and `Direct` wkd types
- Easy to use docker container


## Requirements

- docker-compose
## Configuration

Config name | Accepted values | Meaning
--- | --- | ---
variant | `Advanced` or `Direct` | Use `Advanced` if the keys are accessible on `openpgpkey.yourdomain.tld`; Use `Direct` if the keys are accessible on the same domain (and subdomain) as your email server
max_age | Any number | How long an addition/deletion request can live before becoming stale
cleanup_interval | Any positive number | How much time should pass between stale request cleanups
allowed_domains | Array of strings | What email domains this server should accept
port | Any positive number | Which port the server should bind to
bind_host | An ip address | Which address the server should bind to
external_url | A valid url | The URL to the web interface (this will be used to generate confirmation links) 
mail_settings.smtp_host | String | The SMTP host
mail_settings.smtp_username | String | The username to be used for authentication
mail_settings.smtp_password | String | The password to be used for authentication
mail_settings.smtp_port | Any positive number | The port of the SMTP server
mail_settings.smtp_tls | `Tls` or `Starttls` | The encryption method to use
mail_settings.mail_from | String | The email address to be used
mail_settings.mail_subject | String | The confirmation email's subject
policy.key_max_validity | Any positive number | The maximum allowed validity period of a key in seconds. (optional)


## Environment Variables

You can choose the logging level by setting the `RUST_LOG` environment variable, using the [env_logger](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging) syntax. To filter out logs originating from simple-wkd's dependencies, you should set `RUST_LOG` to `simple_wkd={log_level}`


## Development

To develop the webpage you just need to `cd` to the website directory and run the dev server. This allows to see changes to the code in real-time, but doesn't start the backend.
```bash
$ cd website
$ pnpm install && pnpm dev
```

To develop the backend you need to copy the necessary files (aka. config and assets, including the webpage) to the backend directory, and run the server.
```bash
# Build the webpage
$ cd website
$ pnpm install && pnpm build
$ cd ..
# Copy necessary files to the backend folder
$ cp -r website/dist assets/webpage && cp -r assets backend/assets 
$ cp example.config.toml backend/example.toml 
# Run the server
$ cd backend
$ cargo run
```


## Deployment

1. Download the default `docker-compose.yml` and `config.toml`

    ```bash
    $ wget https://raw.githubusercontent.com/Delta1925/simple-wkd/master/docker-compose.yml
    $ wget https://raw.githubusercontent.com/Delta1925/simple-wkd/master/example.config.toml -O config.toml
    ```

2. Set up simple-wkd by editing the settings in `config.toml` and `docker-compose.yml`.

3. When you're done, start the server using docker-compose

    ```bash
    $ docker-compose up
    ```

This will run the simple-wkd server and publish it on port 8080. You will need to set up a reverse proxy, like [caddy](https://caddyserver.com/). 


## Contributing

Contributions are always welcome!

Please check the issues first to avoid having multiple people work on the same thing and to ensure that your contribution is helpful.

## License

[GNU AGPLv3](https://choosealicense.com/licenses/agpl-3.0/)

