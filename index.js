const mod = require("./index.node");

const obj = mod.create_typed_object('[{"x": 123}, {"x": "jaja"}, {"x": null}]');

console.log("obj", obj);
