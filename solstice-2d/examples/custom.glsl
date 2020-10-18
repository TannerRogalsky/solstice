varying vec2 vPosition;

#ifdef VERTEX
attribute vec4 position;

uniform mat4 uProjection;

void main() {
    vPosition = position.xy;
    gl_Position = uProjection * position;
}
#endif

#ifdef FRAGMENT
void main() {
    vec2 pos = normalize(vPosition);
    fragColor = vec4(pos.x, pos.y, 1. - pos.y, 1.);
}
#endif