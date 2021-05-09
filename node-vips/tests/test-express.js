const express = require('express');
const fs = require('fs');
const addon = require('../index.js')

function sleep(t) {
    return new Promise((r) => setTimeout(r, t))
}

class Vips {
    constructor(vips) {
        this._vips = vips;
    }

    static async create(read_stream) {
        const vips = await new Promise((res, rej) => {
            const res_wrap = (_err, vips) => res(vips);
            const bufferList = new addon.BufferList(10 * read_stream.readableHighWaterMark);

            addon.createVipsImage(res_wrap, rej, bufferList, () => {
                read_stream.on('close', () => {
                    bufferList.close()
                })

                read_stream.on('error', () => {
                    bufferList.close()
                })

                read_stream.on('data', (buf) => {
                    const r = bufferList.push(buf)
                    if (!r)
                        read_stream.pause()
                })
            }, () => {
                read_stream.resume()
            })
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

    async write(write_stream, suffix = ".jpg") {
        write_stream.once('error', (e) => {
            console.log(e)
        })

        write_stream.once('finish', () => {
            //console.log('write finish ' + idx)
        })

        write_stream.once('close', () => {
            //console.log('write close ' + idx)
        })

        return await new Promise((res, rej) => {
            const rej_wrap = (_err, v) => {
                rej(v);
            };

            addon.writeVipsImage(this._vips, suffix, 10 * write_stream.writableHighWaterMark, rej_wrap, async (_err, buf, end) => {
                let r = write_stream.write(buf)
                if (!r) {
                    await new Promise((r) => write_stream.once('drain', () => r()))
                }

                if (end) {
                    res(true)
                }
            });
        });
    }

}

setInterval(() => {
    addon.freeMemory()
    //console.log('free memory')
    //global.gc()
}, 10_000);

const app = express();

app.get('/stream', async (req, res) => {
    const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");
    const vips = await Vips.create(read_stream);
    let r = await vips.write(res, ".jpg");
    res.end();
});

console.log('listening on port 3000')
app.listen({ port: 3000 })
