const addon = require('./index.js')

addon.callTest((_err, ctx) => {
    addon.registerCallTest(ctx)
})
