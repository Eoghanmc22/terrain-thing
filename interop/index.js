const { Vec3 } = require('vec3')

export function place(chunk, x, y, z, block) {
  chunk.setBlockStateId(new Vec3(x, y, z), block);
}