# ansible-inventory-git
This is a little stateless app and plugin for ansible-inventory. It get inventory from some git repo and paste it to sdtout.

## Why
Sometimes you want to use inventory just like role/play. For example, you have several ansible repositories. Each one is independent and you use some CI/CD to run it's code (e.g. Gitlab Runners with `.gitlab-ci.yml` inside each repo). You like to use galaxy collections and other modules from a separate central repository of your company. But what about inventory? It's hard for you to keep your inventory in sync between all those little repositories every time. So why not do the same for inventory? - Just create 1 central inventory repository and call it every time you need to deploy. This application uses this approach.
From this point of view it works exactly the same like other dynamic plugins.

## Requirements

OS:
- linux (arch amd64)
- darwin (apple m1)

Dependencies:

- ansible-inventory should be installed and be able to be called

## Working with

- Tested with ansible version 2.15.2
- Realised only ssh connection to git repo!
- Reuse ansible-inventory -> 100% compability with all ansible versions

## Install

1. Copy this app to some directory.
2. Make sure that you have `script` statement in setting `enable_plugins` in your ansible.cfg file in the [inventory] section.

**Example**
```ini
...
[inventory]
enable_plugins = host_list, script, auto, yaml, ini, toml
...
```

3. Create yaml config file and place it in the same directory as the app. Name of config should be the same as app name.

## Usage

Standalone Example
```bash
ansible-inventory-git -c ./configs/conf.yaml --host lovely-server
```

With ansible like inventory script Example
```bash
ansible -i /some/folder/ans-inv-git lovely_host -m ping
# or use ansible as you always do:
ansible-playbook -i /some/folder/ans-inv-git --diff plays/lovely_play.yml -l lovely_host
```
