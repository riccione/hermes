<a name="readme-top"></a>

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
<div align="center">
  <h3 align="center">Hermes - CLI OTP app</h3>
</div>

<!-- TABLE OF CONTENTS -->
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
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

Simple cli OTP app written in Rust. Mainly for self education and self use.
It uses several crates:
- [https://crates.io/crates/clap](clap)
- [https://crates.io/crates/magic-crypt](magic-crypt)
- [https://crates.io/crates/data-encoding](data-encoding)
- [https://crates.io/crates/totp-lite](totp-lite)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With

* RUST 1.85

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

To build install Rust compiler [https://www.rust-lang.org/](https://www.rust-lang.org/)

For build - run:
`cargo build --release`

For tests - run:
`cargo test -- --test-threads=1`

By default, Rust's testing framework runs tests concurrently for improved performance. 
However, this approach can sometimes cause tests to fail when they're dependent on 
shared resources or are not designed to handle concurrent execution. 
Currently, it seems that certain functionalities (add, update, remove) may not function correctly 
when multiple requests are processed simultaneously.

## TOTP verification
https://authenticationtest.com/totpChallenge/ [https://authenticationtest.com/totpChallenge/](https://authenticationtest.com/totpChallenge/)
https://www.verifyr.com/en/otp/check#totp [https://www.verifyr.com/en/otp/check#totp](https://www.verifyr.com/en/otp/check#totp)
https://totp.danhersam.com/ [https://totp.danhersam.com/](https://totp.danhersam.com/)

### Installation

No installation - portable, one executable file

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->
## Usage

Add new record -> creates a file "codex" in the current directory, 
where alias and encrypted code is saved:

`hermes add --alias <alias> --code <code>`

or

`hermes add -a <alias> -c <code>`

Currently alias should contain only alphanumeric symbols.

Remove record

`hermes remove --alias <alias>`

or

`hermes remove -a <alias>`

Update code for existing alias

`hermes update --alias <alias>`

or

`hermes update -a <alias>`

Get OTP by alias

`hermes ls --alias <alias>`

or

`hermes ls -a <alias>`

Get all OTP codes

`hermes ls`

Show location of codex file, where actually all codes are saved

`hermes config`

By default hermes encrypts code with AES-256, if you want to store code as a
plain text pass `--unencrypt` or `-u`.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

- [ ] Refactor args=clap code
- [ ] Add locks
- [ ] Add unit tests
- [ ] Improve integration tests
- [ ] Refactor code using best Rust practices

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

No reason to contact with me ^_-.
Just create an issue if you need something.

Project Link:
[https://github.com/riccionee/hermes](https://github.com/riccione/hermes)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Choose an Open Source License](https://choosealicense.com)
* [Rust](https://www.rust-lang.org/)

<p align="right">(<a href="#readme-top">back to top</a>)</p>
