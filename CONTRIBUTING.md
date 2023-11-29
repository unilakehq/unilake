# Contributing to Unilake

Unilake is built by an open and friendly community. We are dedicated to building a collaborative, inspiring, and exuberant open-source community for our members. Everyone is more than welcome to join our community to get help and to contribute to Unilake.

## Table of content

- [Contributing to Unilake](#contributing-to-unilake)
  - [Table of content](#table-of-content)
  - [How to contribute](#how-to-contribute)
  - [Contributing guideline](#contributing-guideline)
    - [Report a bug](#report-a-bug)
    - [Contributing code](#contributing-code)
      - [General workflow](#general-workflow)
      - [Unilake code structure](#unilake-code-structure)
      - [Important directories](#important-directories)
      - [Setup your development environment](#set-your-development-environment)
      - [Coding style](#coding-style)
      - [Unit test](#unit-test)
      - [Commit message](#commit-message)
      - [PR body](#pr-body)
      - [Contributor License Agreement](#contributor-license-agreement)
      - [Best practices](#best-practices)
      - [Important contacts](#important-contacts)
    - [Contributing test case](#contributing-test-case)
    - [Reviewing code](#reviewing-code)
    - [Contributing documentation](#contributing-documentation)
    - [Help community members](#help-community-members)
    - [Spread our idea](#spread-our-idea)
  - [Code of conduct](#code-of-conduct)

## How to contribute

Contributions to Unilake are cordially welcome from everyone. Contributing to Unilake is not limited to contributing codes. Below, we list different approaches to contributing to our community.

| Contribution                  | Details                                                                                                                                                                                                   |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Report a bug                  | You can [file an issue](https://github.com/unilakehq/unilake/issues/new/choose) To report a bug with Unilake.                                                                                             |
| Contribute code               | You can contribute your code by fixing a bug or implementing a feature.                                                                                                                                   |
| Contribute test case          | You can contribute your test cases.                                                                                                                                                                       |
| Help review code              | If you are an active contributor or committer of Unilake, you can help us review the pull requests (PRs).                                                                                                 |
| Contribute documentation      | Unilake community maintains a tremendous amount of documentation both in Chinese and English. You can contribute documentation changes by fixing a documentation bug or proposing a new piece of content. |
| Help Unilake users            | You can help newcomers who meet difficulties in our community.                                                                                                                                            |
| Spread the word about Unilake | You can author an article or give a talk about us to help us spread our technology to the world.                                                                                                          |

## Contributing guideline

This guide describes how to make various types of contribution to Unilake community.

### Report a bug

To report a bug with Unilake, you should [file an issue](https://github.com/unilakehq/Unilake/issues/new/choose) in Unilake repository, and provide necessary information and, if possible, steps to reproduce the bug in the issue body.

### Contributing code

You can contribute your code by fixing a bug you identified or in an [existing issue](https://github.com/unilakehq/Unilake/issues). If you are new to this project, you may find the issues labelled good-first-issue suitable for your first contribution. Usually, such issues provide a detailed description of the procedure to solve the problem in its issue body.

If you are confident with your programming proficiency, you can also contribute your code by helping develop a feature for Unilake.

#### General workflow

Before getting your hands on codes, you should comment and mention the repository maintainer in the issue body, and inform him/her to assign to you the issue that you wish to solve. It is recommended to share your plan on how to solve this problem in the issue body as well.

In Unilake community, we follow the fork-and-merge GitHub workflow when contributing code.

1. Create a fork of Unilake in your GitHub account.
2. Clone this forked repository to your local device.
3. Check out a new branch based on the branch you expect to contribute.
4. Commit your code changes on the new branch.
5. Push the branch with code changes to GitHub.
6. Create a PR to submit your code changes.

The repository maintainers will review your code changes as soon as possible. Your commits will be merged once approved.

For detailed instruction on GitHub workflow, see [Unilake GitHub Workflow](https://github.com/unilakehq/community/blob/main/Contributors/guide/workflow.md).

#### Unilake code structure

TBC

#### Important directories

TBC

#### Set-up your development environment

In order to accelerate the development of new features, we have created a devcontainer with a lot batteries included. When developing in the devcontainer the following process resembles a common way of work:

- After opening the devcontainer, adjust the unilake.development.yaml file in the root of the project. Adjust it in a way so that the service you are working on is not deployed to the target kubernetes cluster and dependent services connect to the version you are developing instead.
- Run `k3d cluster create --api-port 6550 -p "8081:80@loadbalancer" --agents 1` to create a local kubernetes cluster
- Run `k3d kubeconfig merge k3s-default --kubeconfig-merge-default --kubeconfig-switch-context` to make sure your kubeconfig is updated
- Run `unilake up` in the project's root directory to launch the cluster
- After running the above command, you can make use of `sudo --preserve-env kubefwd svc -n default` to link services to your local development environment

#### Coding style

TBC

#### Unit test

TBC

#### Commit message

- Write your commit message in English.
- Start your commit message with a verb clause initiated with an upper case letter.
- Use concise, explicit language.

#### PR body

- You should relate the issue you worked on in the PR body.
- It is recommended to submit ONE commit in ONE PR.
See [PR template](https://github.com/unilakehq/unilake/blob/main/.github/PULL_REQUEST_TEMPLATE.md) for more inforamtion.

#### Contributor License Agreement

To get your PR merged, you must [submit your Contributor License Agreement (CLA)](https://cla-assistant.io/unilakehq/unilake) first. You only need to submit it ONCE.

#### Best practices

TBC

#### Important contacts

Whenever you have difficulties in development, you can reach out to the following members for help. You can mention them in your issue or PR.

- TBC

### Contributing test case

TBC

### Reviewing code

TBC

### Contributing documentation

TBC

### Help community members

If you are a proficient Unilake user, you can contribute to our community by helping new members solve the problems when they use Unilake.

### Spread our idea

You are welcome to author a blog article about Unilake in the media, or host a live broadcast to spread Unilake to the world. Please contact oss@unilake.com for more information and instructions.

## Code of conduct

All members in the community are instructed to follow our [Code of Conduct](https://github.com/unilakehq/unilake/blob/main/CODE_OF_CONDUCT.md).