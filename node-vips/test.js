const addon = require('./index.js')
const fs = require('fs');
const memwatch = require('@airbnb/node-memwatch');


async function test(idx) {
    const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");

    let vips = await new Promise((res, rej) => {
        const res_wrap = (_err, v) => res(v);

        let count = 0
        addon.createVipsImage(res_wrap, rej, async (err, ctx, read_size) => {
            if (read_stream.readableEnded) {
                addon.registerReadBuf(ctx, null)
                return
            }

            let buf = await new Promise((res) => read_stream.once('readable', () => {
                const b = read_stream.read(read_size)
                res(b)
            }))

            count += buf.length
            addon.registerReadBuf(ctx, buf)
        })
    })

    //console.log('vips = ', vips)

    const write_stream = fs.createWriteStream(`/tmp/test/thumb${idx}.jpg`);
    let r = await new Promise((res, rej) => {
        const res_wrap = (_err, v) => {
            write_stream.end(() => {
                res(v);
            })
        }

        addon.writeVipsImage(vips, ".jpg", res_wrap, rej, async (err, ctx, buf, mystruct) => {
            if (write_stream.writableEnded) return

            let r = write_stream.write(buf)
            if (!r) {
                await new Promise((r) => write_stream.once('drain', r))
                console.log('drain')
            }

            addon.registerWriteSize(ctx, buf.length)
        });
    });

    //console.log(r)
}

function showMemUsage() {
    const used = process.memoryUsage();
    for (let key in used) {
        console.log(`${key} ${Math.round(used[key] / 1024 / 1024 * 100) / 100} MB`);
    }
}

function showMemStats() {
    const used = addon.getMemStats()
    for (let key in used) {
        console.log(`${key} ${Math.round(used[key] / 1024 / 1024 * 100) / 100} MB`);
    }
}

(async () => {
    global.gc()
    console.log('start ' + process.pid)
    //await new Promise((r) => setTimeout(r, 10000))
    const hd = new memwatch.HeapDiff();
    showMemUsage();
    showMemStats()
    let proms = [];
    for (let i = 0; i < 50; i++) {
        proms.push(test(i))
        //showMemStats()
        //showMemUsage();
        //console.log('=====================')
    }

    await Promise.all(proms)
    global.gc()
    showMemStats()
    console.log('=====================')
    showMemUsage();
    console.log('end')
    addon.shutdown()
    global.gc()
    const diff = hd.end();
    console.log("memwatch diff:", JSON.stringify(diff, null, 2));
    showMemStats()
    showMemUsage();
    //await new Promise((r) => setTimeout(r, 3000))
})();
