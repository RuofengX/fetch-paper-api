# fetch-paper-api
It is a simple python script to download jar file from papermc.io/api/v2 with customable project(paper, velocity, etc.) and version(1.18.1, 3.1.1, etc.)
## Brief Guide
### Requirements
- python3
- modules listed in requirements.txt, use `pip install -r requirements.txt` to install all of them
### Usage
`python fetch-paper-api.py -p <project> -v <version>`  
Target file will save to disk named with 'target.jar' and a validate file named with 'target.jar.sha256'
### Example  
> I want to download latest build of papermc 1.18.1, using:  
`python fetch-paper-api.py -p paper -v 1.18.1`

> I want to download latest build of velocity 3.1.1, using:  
`python fetch-paper-api.py -p velocity -v 3.1.1`

## Docker Example
Please refer to Dockerfile in this repository.
