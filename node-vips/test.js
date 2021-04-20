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

    //const write_stream = fs.createWriteStream(`/tmp/test/thumb${idx}.jpg`);
    const write_stream = fs.createWriteStream('/dev/null');
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

let cancel_token = setInterval(() => {
    console.log('free memory ' + addon.freeMemory())
}, 1000);

(async () => {
    console.log('start ' + process.pid)
    //const hd = new memwatch.HeapDiff();
    showMemUsage();
    console.log('=====================')
    showMemStats()
    let proms = [];
    for (let i = 0; i < 100; i++) {
        proms.push(test(i))
    }

    await Promise.all(proms)
    //const diff = hd.end();
    //console.log("memwatch diff:", JSON.stringify(diff, null, 2));
    console.log('free memory ' + addon.freeMemory())
    await new Promise((r) => setTimeout(r, 100))
    console.log('=====================')
    showMemUsage();
    console.log('=====================')
    showMemStats()
    clearInterval(cancel_token)
    //await new Promise((r) => setTimeout(r, 3000))
})();
