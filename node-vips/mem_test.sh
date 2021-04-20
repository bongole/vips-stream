#!/bin/bash

time UV_THREADPOOL_SIZE=10 MALLOC_ARENA_MAX=2 node --expose-gc test.js
#time LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so.2 node --expose-gc test.js
