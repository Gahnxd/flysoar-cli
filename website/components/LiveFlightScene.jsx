"use client";

import { useEffect, useRef } from "react";

export default function LiveFlightScene() {
  const routesRef = useRef(null);

  useEffect(() => {
    let disposed = false;
    let renderer;
    let animationFrame;
    let resizeObserver;
    let removePointerListener;
    let timer;
    let routeGroup;
    let signalTexture;

    async function initializeScene() {
      const THREE = await import("three");
      if (disposed || !routesRef.current) return;

      const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

      const routeHost = routesRef.current;
      const scene = new THREE.Scene();
      const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 10);
      camera.position.z = 2;
      renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
      renderer.setClearColor(0x000000, 0);
      renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
      routeHost.appendChild(renderer.domElement);

      routeGroup = new THREE.Group();
      scene.add(routeGroup);

      const signalCanvas = document.createElement("canvas");
      signalCanvas.width = 64;
      signalCanvas.height = 64;
      const signalContext = signalCanvas.getContext("2d");
      const signalGradient = signalContext.createRadialGradient(32, 32, 0, 32, 32, 32);
      signalGradient.addColorStop(0, "rgba(255,255,255,1)");
      signalGradient.addColorStop(0.22, "rgba(188,211,255,0.95)");
      signalGradient.addColorStop(0.55, "rgba(111,151,255,0.38)");
      signalGradient.addColorStop(1, "rgba(80,120,255,0)");
      signalContext.fillStyle = signalGradient;
      signalContext.fillRect(0, 0, 64, 64);
      signalTexture = new THREE.CanvasTexture(signalCanvas);

      const hubs = {
        sfo: [-0.61, -0.46],
        nyc: [-0.4, -0.39],
        sao: [-0.25, -0.7],
        lon: [0.06, -0.37],
        cai: [0.2, -0.49],
        dxb: [0.35, -0.48],
        sin: [0.63, -0.58],
        tyo: [0.75, -0.43],
        syd: [0.84, -0.7],
      };
      const routePairs = [
        ["sfo", "nyc"], ["sfo", "lon"], ["sfo", "tyo"],
        ["nyc", "lon"], ["nyc", "sao"], ["lon", "dxb"],
        ["lon", "sin"], ["cai", "tyo"], ["dxb", "sin"],
        ["sin", "syd"], ["tyo", "syd"],
      ];
      const colors = [0x7d92ff, 0x69c9ff, 0xa47cff];
      const routeItems = routePairs.map(([from, to], index) => {
        const [x1, y1] = hubs[from];
        const [x2, y2] = hubs[to];
        const distance = Math.abs(x2 - x1);
        const lift = 0.14 + distance * 0.38;
        const curve = new THREE.QuadraticBezierCurve3(
          new THREE.Vector3(x1, y1, 0),
          new THREE.Vector3((x1 + x2) / 2, Math.max(y1, y2) + lift, 0),
          new THREE.Vector3(x2, y2, 0)
        );
        const lineGeometry = new THREE.BufferGeometry().setFromPoints(curve.getPoints(72));
        const lineMaterial = new THREE.LineBasicMaterial({
          color: colors[index % colors.length],
          transparent: true,
          opacity: 0.28,
          blending: THREE.AdditiveBlending,
          depthWrite: false,
        });
        const line = new THREE.Line(lineGeometry, lineMaterial);
        routeGroup.add(line);

        const signalGeometry = new THREE.BufferGeometry();
        signalGeometry.setAttribute("position", new THREE.Float32BufferAttribute([x1, y1, 0], 3));
        const signalMaterial = new THREE.PointsMaterial({
          color: 0xdbe5ff,
          map: signalTexture,
          size: index % 3 === 0 ? 10 : 7,
          sizeAttenuation: false,
          transparent: true,
          opacity: 0.95,
          alphaTest: 0.01,
          blending: THREE.AdditiveBlending,
          depthWrite: false,
        });
        const signal = new THREE.Points(signalGeometry, signalMaterial);
        routeGroup.add(signal);

        return {
          curve,
          lineMaterial,
          signalGeometry,
          phase: index / routePairs.length,
          speed: 0.045 + (index % 4) * 0.009,
        };
      });

      function resize() {
        renderer.setSize(routeHost.clientWidth, routeHost.clientHeight, false);
      }
      resizeObserver = new ResizeObserver(resize);
      resizeObserver.observe(routeHost);
      resize();

      const pointerTarget = new THREE.Vector2();
      function onPointerMove(event) {
        pointerTarget.set(
          (event.clientX / window.innerWidth - 0.5) * 0.025,
          (event.clientY / window.innerHeight - 0.5) * -0.018
        );
      }
      window.addEventListener("pointermove", onPointerMove, { passive: true });
      removePointerListener = () => window.removeEventListener("pointermove", onPointerMove);

      timer = new THREE.Timer();
      timer.connect(document);
      function render(timestamp) {
        timer.update(timestamp);
        const elapsed = timer.getElapsed();
        routeGroup.position.x += (pointerTarget.x - routeGroup.position.x) * 0.035;
        routeGroup.position.y += (pointerTarget.y - routeGroup.position.y) * 0.035;

        routeItems.forEach((item, index) => {
          const progress = (elapsed * item.speed + item.phase) % 1;
          const point = item.curve.getPointAt(progress);
          const positions = item.signalGeometry.attributes.position;
          positions.setXYZ(0, point.x, point.y, point.z);
          positions.needsUpdate = true;
          item.lineMaterial.opacity = 0.23 + Math.sin(elapsed * 0.8 + index) * 0.075;
        });

        renderer.render(scene, camera);
        if (!reducedMotion) animationFrame = window.requestAnimationFrame(render);
      }
      render();
    }

    initializeScene().catch((error) => {
      console.error("Unable to initialize animated background", error);
    });

    return () => {
      disposed = true;
      if (animationFrame) window.cancelAnimationFrame(animationFrame);
      if (resizeObserver) resizeObserver.disconnect();
      if (removePointerListener) removePointerListener();
      if (timer) timer.dispose();
      if (routeGroup) {
        routeGroup.traverse((object) => {
          if (object.geometry) object.geometry.dispose();
          if (object.material) object.material.dispose();
        });
      }
      if (signalTexture) signalTexture.dispose();
      if (renderer) {
        renderer.dispose();
        renderer.domElement.remove();
      }
    };
  }, []);

  return (
    <>
      <div className="flight-static-image" />
      <div className="three-routes" ref={routesRef} />
    </>
  );
}
