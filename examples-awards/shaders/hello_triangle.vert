#version 420 core

const vec2 VERTEX_POSITIONS[3] = {
    vec2(-0.5, -0.5), 
    vec2( 0.5, -0.5), 
    vec2( 0.0,  0.5)
};

void main() {
    gl_Position = vec4(VERTEX_POSITIONS[gl_VertexID], 0.5, 1.0);
}
