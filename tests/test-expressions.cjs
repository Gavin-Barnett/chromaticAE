const assert = require('node:assert/strict');
const vm = require('node:vm');
const { positionExpression, quote } = require('../Chromatic AE.jsx');
let assertions = 0;
function position(channel, values) {
    const controls = { 'Split Amount (%)': 100, 'Direction': 0,
        'Red Distance (px)': -8, 'Green Distance (px)': 0, 'Blue Distance (px)': 8,
        'Red XY Offset': [0,0], 'Green XY Offset': [0,0], 'Blue XY Offset': [0,0], ...values };
    return Array.from(vm.runInNewContext(positionExpression('var c = controller;\n', channel), {
        controller: { effect: name => () => { assert.ok(name in controls); return controls[name]; } },
        thisComp: { width:1920, height:1080 }, Math
    }));
}
function close(actual, expected) { assert.equal(actual.length, expected.length); actual.forEach((v,i)=>assert.ok(Math.abs(v-expected[i])<1e-8, `${actual} != ${expected}`)); assertions++; }
close(position('Red'), [952,540]);
close(position('Green'), [960,540]);
close(position('Blue'), [968,540]);
close(position('Red', {'Direction':90}), [960,532]);
close(position('Blue', {'Direction':180}), [952,540]);
close(position('Green', {'Green Distance (px)':7.25, 'Green XY Offset':[-2,3]}), [965.25,543]);
for (const channel of ['Red','Green','Blue']) {
    close(position(channel, {'Split Amount (%)':0, [channel+' XY Offset']:[900,-600]}), [960,540]);
    const p=position(channel, {'Split Amount (%)':-100});
    const q=position(channel);
    close([p[0]+q[0],p[1]+q[1]],[1920,1080]);
}
for (const name of ['hello','name "quoted"','slash\\name','line\nreturn\r']) { assert.equal(vm.runInNewContext(quote(name)),name); assertions++; }
console.log(`${assertions} expression assertions passed.`);
