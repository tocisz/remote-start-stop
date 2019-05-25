# Remote start stop

This tool allows simply starting / stopping remote service through SSH and opening web browser.

## Configuration

Application reads config.yml file from current directory. An example:

```yaml
ssh:
  host: "host:22" # where to connect
  username: "user" # username
  key: "key.pem" # path to private key file
  commands: # map of arbitrary commands
    start:
      command: "service x start"
    stop:
      command: "service x stop"
    status:
      command: "service x status"
      expected: "OK" # checking for expected string not implemented yet
opener:
  link: "https://host:8080/"
```

With this config file you can use an application to run commands remotely:
 * `remote start open` -- will execute `service x start` through ssh and open `https://host:8080/` in a browser;
 * `remote stop` -- will execute `service x stop` through ssh.
 
 