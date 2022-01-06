# fetch-paper-api
It is a simple python script to download jar file from papermc.io/api/v2 with customable project(paper, velocity, etc.) and version(1.18.1, 3.1.1, etc.)
## Brief Guide
### Requirements
- python3
- modules listed in requirements.txt, use `pip install -r requirements.txt` to install all of them

### Example(TL;NR)  
> I want to download build 100 of papermc 1.18.1, using:  
`python fetch-paper-api.py paper -v 1.18.1 -b 100`

> I want to download latest build of velocity 3.1.1, using:  
`python fetch-paper-api.py velocity -v 3.1.1`

### Usage
```
usage: fetch-paper-api.py [-h] [-b BUILD] project version

Fetch latest build from paper.io/api/v2, and check them with SHA256.

positional arguments:
  project               choice which project should use.
  version               choice which version should use

optional arguments:
  -h, --help            show this help message and exit
  -b BUILD, --build BUILD
                        build number, leave blank to fetch latest.

```

## Docker Example
Please refer to Dockerfile in this repository.
