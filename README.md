# SJTU-HPC-server

This is a tool , which is written in rust, to help you upload your code files to the SJTU HPC, and get the result from that.

## Getting Started

Download links:

SSH clone URL: ssh://git@github.com:Yuan-Allen/SJTU-HPC-server.git

HTTPS clone URL: https://github.com/Yuan-Allen/SJTU-HPC-server.git

These will get you a copy of the project up and running on your local machine for development and testing purposes.

## Prerequisites

- You need to prepare your code files which is going to upload to the SJTU HPC.
- You also need to have an account to the SJTU HPC in order to successfully connect. 

## Deployment

There is an Dockerfile example in it, which can make it a base image to help you build your own task images. (It's used in [rMinik8s](https://github.com/markcty/rMiniK8s))

Or you can just directly run this program with environment variables.

## Usage

You need to sse environment variables to specify these parameters.
- Required
  - USERNAME: Your HPC username.
  - PASSWORD: Your HPC password.
  - JOB_NAME: The job name you want to specify.
- Optional
  - ACCOUNT: Your HPC account. (Default: acct-stu)
  - RESOURCE_DIR: All files in this directory will be upload to HPC, and the result will also be saved in it. (Default: /code)
  - CODE_FILE_NAME: The file name of you code file, which should be a `zip` file. (Default: gpu.zip)
  - REMOTE_DIR: The remote directory in the HPC you want to upload to. (Default: /lustre/home/ACCOUNT/USERNAME/JOB_NAME)
  - COMPILE_SCRIPT: The compile script. It will be executed in REMOTE_DIR once you finished to upload and unzip in REMOTE_DIR. (Default: make)

E.g.
```shell
USERNAME=testuser PASSWORD=123456 JOB_NAME=cuda ./gpu_server
```

## License
Distributed under the [GPL License](https://www.gnu.org/licenses/gpl-3.0.html).
