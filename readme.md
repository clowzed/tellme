<div align="center">
    <img src="https://cdn-icons-png.flaticon.com/512/44/44386.png" alt="Logo" width="80" height="80">

  <h3 align="center">Tellme</h3>

  <p align="center">
  Tiny service registry for MVP and microservices
  </p>
</div>



<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
        <li><a href="#usage">Usage</a></li>
        <li><a href="#running">Running</a></li>
        <li><a href="#for-running">For running</a></li>
      </ul>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>



## About The Project
If you need fast service registry without pain - use this (not for production)


### Built With


* [actix-web](https://actix.rs/)
* [Rust](https://rust.org/)




## Getting Started
To get a local copy up and running follow these simple example steps.

### Prerequisites

You must have `rust` and `cargo` installed.

### Installation

1. Clone the repo
   ```sh
   git clone https://github.com/clowzed/tellme.git
   ```
2. build executable
   ```sh
   cargo build --release
   ```

### Running

| short | long       | description                          | required | default      |
| ----- | ---------- | ------------------------------------ | -------- | ------------ |
| -i    | --interval | Sets healthcheck interval in seconds | false    | 30           |
| -p    | --port     | Sets port for server                 | flase    | 5000         |
| -c    | --creds    | Sets filename for credentials        | false    | tellme.creds |
| -h    | --help     | Print usage information              | false    |              |

#### For running
```sh
cargo run --release -- <params here>
```

### Usage

1) On server startup `tellme.creds` file is created with login and password for access
2) If you want to register service you firstly need access token.
```python
import request

response = request.post("{server_ip}/newtoken", data = dict(login    = login_from_file,
                                                            password = password_from_file))

if response.status_code == 200:
  print("Access token: ", response.json().get("token", None))
```
3) Then you need to send request to server with access token and params
Server will recive you your unique identifier

| param                | decription                                   |
| -------------------- | -------------------------------------------- |
| access_token         | Token given in last request                  |
| healthcheck_endpoint | Point for ping service to check availability |
| service_type         | Type of the service for query in /find       |
```python
import request

response = request.post("{server_ip}/me", data = dict(access_token         = access_token,
                                                      healthcheck_endpoint = "{anyserver}/{any route}",
                                                      service_type         = "storage node"))
if response.status_code == 200:
  identifier = response.json().get("identifier", None)

```


4) Now your service is not shown to any other clients. You need to accept it.
```python
import request

response = request.post("{server_ip}/accept", data = dict(login      = login_from_file,
                                                          password   = password_from_file,
                                                          identifier = identifier))
if response.status_code = 202:
  print("Service was successfully accepted!")

```

5) Now it is available and shown to others.
You can get information about services using /find


| param        | decription                                                              |
| ------------ | ----------------------------------------------------------------------- |
| limit        | limits the amount of services to be returned  `usize`                   |
| is_available | returns services which are now available. Default: returns all services |
| service_type | type of the services to filter on. Default: all types are returned      |

```python
import request

response = request.get("{server_ip}/find", params = dict()
if response.status_code = 200:
  print("Services: ", response.json())

```


## Roadmap

- [ ] Add ping service (mocked now)
- [ ] SSL Support
- [ ] Cli tool for admins (tellme-cli)




## Contributing
If you have a suggestion that would make this better, please fork the repo and create a pull request.
You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/feature`)
3. Commit your Changes (`git commit -m 'Add some feature'`)
4. Push to the Branch (`git push origin feature/feature`)
5. Open a Pull Request



## License

Distributed under the MIT License. See `LICENSE` for more information.



## Contact

Me - [@clowzed](https://vk.com/clowzed) - [clowzed.work@gmail.com](mailto:clowwzed.work@gmail.com)

Project - [https://github.com/clowzed/tellme](https://github.com/clowzed/tellme)



