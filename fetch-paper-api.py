import json
import sys
import logging
import urllib.request
import argparse
import requests
import time

PAPER_API_URL = "https://papermc.io/api/v2"

# logger init
logger = logging.getLogger()    # initialize logging class
logger.setLevel(logging.WARN)  # default log level,
format = logging.Formatter("%(asctime)s - %(message)s")    # output format
sh = logging.StreamHandler(stream=sys.stdout)    # output to standard output
sh.setFormatter(format)
logger.addHandler(sh)

# argument parser
parser = argparse.ArgumentParser()
parser.add_argument("-p", "--project", type=str,
                    help=f"Choice which project should use. Should be ")
parser.add_argument("-v", "--version", type=str,
                    help="Choice which version should use")
args = parser.parse_args()

# Quick entry
open_url = urllib.request.urlopen

# Custom errors


class NetworkError(Exception):
    def __init__(self, url, code):
        self.code = code
        self.url = url
        logger.error(f"Network error with code {self.code} when opening {url}")

class EntryNotExistError(Exception):
    def __init__(self, content, group, avil):
        logger.error(f"Target {content} is not in {group} list, valid choice on api now is: {avil}")

# Main code below
# From here it is expected to create a link like these:
# `https://papermc.io/api/v2/projects/waterfall/versions/1.16/builds/430/downloads/waterfall-1.16-430.jar`
# it could be structed like:
# {PAPER_API_URL}/projects/{project}/versions/{version}/builds/{build}/downloads/{app_name}
# {app_name} is auto fetched, and each part of this url is parsed by one class.


class Link():
    def __init__(self):
        self.base = PAPER_API_URL
        self.link = self.base

    def safe_open(cls, url: str):
        logger.debug(f'Safe open url: {url}')
        _ret = open_url(url)
        if _ret.code != 200:
            raise NetworkError(url, _ret.code)
        else:
            return _ret.read()


class Projects(Link):
    def __init__(self):
        super().__init__()
        logger.warning(f'Start checking and fetching, please wait.')
        self.link = self.link + "/projects"
        self.project_list = self.project_json_parse(self.link)


    def project_json_parse(self, url):
        _ret = self.safe_open(url)
        _list = json.loads(_ret)['projects']
        logger.debug(f'Avaliable projects is:{_list}')
        return _list


class Versions(Projects):
    def __init__(self, project):
        super().__init__()
        if project in self.project_list:
            logger.info(f'Project: {project} is aviliable')
        else:
            raise EntryNotExistError(
                content=project,
                group='project',
                avil=self.project_list)
        self.project = project



        self.link = self.link + f"/{self.project}"
        self.version_list = self.version_json_parse(self.link)

    def version_json_parse(self, url):
        _ret = self.safe_open(url)
        _list = json.loads(_ret)['versions']
        logger.debug(f'Avaliable versions is:{_list}')
        return _list


class Builds(Versions):
    def __init__(self, project, version):
        super().__init__(project)
        if version in self.version_list:
            logger.info(f'Version: {version} is aviliable')
        else:
            raise EntryNotExistError(
                content=version,
                group='version',
                avil=self.version_list)

        self.version = version
        self.link = self.link + f"/versions/{self.version}"
        self.build_list = self.build_json_parse(self.link)

    def build_json_parse(self, url):
        _ret = self.safe_open(url)
        _list = json.loads(_ret)['builds']
        logger.debug(f'Avaliable builds is:{_list}')
        return _list


class Downloads(Builds):
    def __init__(self, project, version, build):
        super().__init__(project, version)
        # Handle if target is latest or a number
        if build == 'latest':
            logger.debug(f'Using latest build{self.get_latest_build()}')
            self.build = self.get_latest_build()
        elif isinstance(build, int):
            if build in self.build_list:
                logger.info(
                    f'Input build number is valid. Using latest build{self.get_latest_build()} instead!')
                self.build = self.get_latest_build()
            elif build != self.get_latest_build():
                logger.info(
                    f'Input build is not latest, still try to download...')
                self.build = build
            else:
                logger.warning(
                    f'Input build number is NOT listed on {self.link}. Using latest build{self.get_latest_build()} instead!')
                self.build = self.get_latest_build()
        else:
            logger.warning(
                f'Unexpected build number: {self.build}, use latest build{self.get_latest_build()} insteat')
            self.build = self.get_latest_build()

        self.link = self.link + f"/builds/{self.build}"

    def get_latest_build(self):
        _latest_num = self.build_list[-1]
        logger.debug(f"Latest build is {_latest_num}")
        return _latest_num


class Application(Downloads):
    def __init__(self, project, version, build):
        super().__init__(project, version, build)
        self.app_name, self.valid = self.app_json_parse(self.link)
        logger.debug(f'name: {self.app_name}, sha256: {self.valid}')
        self.download_link = self.link + f'/downloads/{self.app_name}'

    def app_json_parse(self, url):
        _ret = self.safe_open(url)
        _json = json.loads(_ret)
        _json = _json['downloads']['application']
        # now json is like: {'application': {'name': 'paper-1.18.1-133.jar', 'sha256': '256f54f8fc984433be0d7f204cda72500aa4e20a59b0ae0324a0978f785c8af1'}

        _name, _sha256 = _json['name'], _json['sha256']

        return _name, _sha256

    def download_file(self):
        logger.warning(
            f'Target Project: {self.project} , Version: {self.version} , Build: {self.build}, Application name: {self.app_name}')
        logger.warning(f'SHA256 code is {self.valid}')
        logger.warning(f'Download begin')

        try:
            _file = requests.get(self.download_link)
            logger.warning(f'Download Successed.')
        except urllib.error.HTTPError:
            logger.error(f'HTTP Error')
            return False

        logger.info(f'Download finished, write into file...')

        # Use project name instead of file, you may change filename here.
        _filename = f'target.jar'
        with open(_filename, 'wb') as _:
            _.write(_file.content)
        with open(f'{_filename}.sha256', 'w') as _:
            _.write(self.valid)
        logger.warning(f'File is saved as {_filename}.')
        logger.info(f'File SHA256 validate code is {self.valid}')
        return True


if __name__ == '__main__':
    app = Application(
        project=args.project,
        version=args.version,
        build='latest'  # a number: int is also OK
    )
    app.download_file()
