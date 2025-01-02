import { Object3DNode } from '@react-three/fiber';
import { AmbientLight, Mesh, PointLight, SphereGeometry } from 'three';

declare module '@react-three/fiber' {
    interface ThreeElements {
        ambientLight: Object3DNode<AmbientLight, typeof AmbientLight>;
        pointLight: Object3DNode<PointLight, typeof PointLight>;
        mesh: Object3DNode<Mesh, typeof Mesh>;
        sphereGeometry: Object3DNode<SphereGeometry, typeof SphereGeometry>;
    }
} 