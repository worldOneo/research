vec3 quaternion_rotate(vec4 q, vec3 v) {
    vec3 tmp = cross(q.xyz, v) + q.w * v;
    vec3 rotated = v + 2.0 * cross(q.xyz, tmp);
    return rotated;
}

vec4 quaternion_create(vec3 axis, float angle) {
    float c = cos(angle * .5);
    float s = sin(angle * .5);
    return normalize(vec4(axis.xyz * s, c));
}
