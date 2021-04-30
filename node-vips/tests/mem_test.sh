#!/bin/bash

#time node --expose-gc test.js
time UV_THREADPOOL_SIZE=20 node --expose-gc test.js
#time LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so.2 node --expose-gc test.js
