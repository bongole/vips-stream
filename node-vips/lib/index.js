var addon = require('../native');

addon.buffer_check(Buffer.from([1,2,3]))
//addon.thread_test()
console.log("before vips_new")
addon.vips_new()
