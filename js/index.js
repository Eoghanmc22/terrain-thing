const { WorldView, Viewer, MapControls } = require('prismarine-viewer/viewer')
const { Vec3 } = require('vec3')
global.THREE = require('three')

async function main () {
  const terrain = await require('terrain')
  const viewDistance = 10
  const config = terrain.init(terrain.SlopeMode.VonNeumann, viewDistance)

  const version = '1.18';

  const World = require('prismarine-world')(version)
  const Chunk = require('prismarine-chunk')(version)

  const world = new World((chunkX, chunkZ) => {
    const chunk = new Chunk();

    const start = performance.now()

    terrain.build_chunk(chunk, chunkX, chunkZ, config)

    const time = performance.now() - start
    console.log(`Generated chunk x: ${chunkX} z: ${chunkZ} in ${time} ms`)

    return chunk;
  });

  const center = new Vec3(0, 100, 0)

  const worldView = new WorldView(world, viewDistance, center)

  // Create three.js context, add to page
  const renderer = new THREE.WebGLRenderer()
  renderer.setPixelRatio(window.devicePixelRatio || 1)
  renderer.setSize(window.innerWidth, window.innerHeight)
  document.body.appendChild(renderer.domElement)

  window.addEventListener('resize', function () {
    viewer.camera.aspect = window.innerWidth / window.innerHeight;
    viewer.camera.updateProjectionMatrix();

    renderer.setSize(window.innerWidth, window.innerHeight);
  }, false);

  // Create viewer
  const viewer = new Viewer(renderer)
  viewer.setVersion(version)
  // Attach controls to viewer
  const controls = new MapControls(viewer.camera, renderer.domElement)
  controls.translateY(75)

  // Link WorldView and Viewer
  viewer.listen(worldView)
  // Initialize viewer, load chunks
  await worldView.init(center)

  viewer.camera.position.set(center.x, center.y, center.z)
  controls.update()

  await viewer.waitForChunksToRender()

  // Browser animation loop
  const animate = () => {
    window.requestAnimationFrame(animate)
    if (controls) controls.update()
    worldView.updatePosition(controls.target)
    viewer.update()
    renderer.render(viewer.scene, viewer.camera)
  }
  animate()
}
main()