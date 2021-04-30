const express = require('express');
const fs = require('fs');
const addon = require('../index.js')

function sleep(t) {
    new Promise((r) => setTimeout(r, t))
}

class Vips {
    constructor(vips){
        this._vips = vips;
    }

    static async create(read_stream) {
        const vips = await new Promise((res, rej) => {
            const res_wrap = (_err, vips) => res(vips);
            const bufferList = new addon.BufferList(3 * read_stream.readableHighWaterMark);

            addon.createVipsImage(res_wrap, rej, bufferList, () => {
                read_stream.on('close', () => {
                    bufferList.close()
                })

                read_stream.on('error', () => {
                    bufferList.close()
                })

                read_stream.on('data', (buf) => {
                    const r = bufferList.push(buf)
                    if( !r )
                        read_stream.pause()
                })
            }, () => read_stream.resume())
        })

        return new Vips(vips)
    }

    thumbnail(width) {
        addon.thumbnail(this._vips, width)
        return this
    }

    resize(vscale) {
        addon.resize(this._vips, vscale)
        return this
    }

    async write(write_stream, suffix = ".jpg", idx) {
        write_stream.once('error', () => {
            console.log('write error ' + idx)
        })

        write_stream.once('finish', () => {
            //console.log('write finish ' + idx)
        })

        write_stream.once('close', () => {
            //console.log('write close ' + idx)
        })

        return await new Promise((res, rej) => {
            const res_wrap = (_err, v) => {
                res(v);
            };

            addon.writeVipsImage(this._vips, suffix, res_wrap, rej, async (err, ctx, buf, mystruct) => {
                if (!write_stream.writable) {
                    addon.registerWriteSize(ctx, -1)
                    return
                }

                let r = write_stream.write(buf)
                if (r) {
                    addon.registerWriteSize(ctx, buf.length)
                } else {
                    console.log('before drain ' + idx)
                    let r = await Promise.race([new Promise((r) => write_stream.once('drain', () => r('drain'))), sleep(1000)])
                    if (r === 'drain') {
                        //write_stream.removeAllListeners();
                        //console.log('after drain writable ' + idx)
                        addon.registerWriteSize(ctx, buf.length)
                    } else {
                        console.log('after drain not writable ' + idx)
                        addon.registerWriteSize(ctx, -1)
                    }
                }
            });
        });
    }

}

setInterval(() => {
    //addon.freeMemory()
    //console.log('free memory')
    //global.gc()
}, 1_000);

const app = express();
let id = 0;
function format(n) {
    return ('000' + n).slice(-3);
}

app.get('/stream', async (req, res) => {
    let myid = ++id;
    console.log('write highwatermark' + res.writableHighWaterMark)
    const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg", { highWaterMark: 40 * 1024 });
    const vips = await Vips.create(read_stream);
    //console.log('start ' + format(myid))
    //let r = await vips.resize(0.109).write(res, ".jpg", format(myid));
    let r = await vips.write(res, ".jpg", format(myid));
    //console.log('end ' + format(myid) + ' ' + r)
    res.end();
});

console.log('listening on port 3000')
app.listen({ port: 3000 })