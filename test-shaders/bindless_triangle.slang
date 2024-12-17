[[vk::binding(0)]]
ByteAddressBuffer uniform_buffers[];
[[vk::binding(1)]]
RWTexture2D<float4> storage_images[];
[[vk::binding(2)]]
Sampler2D<float4> sampled_images[];
[[vk::binding(3)]]
ByteAddressBuffer storage_buffers[];
[[vk::binding(4)]]
RaytracingAccelerationStructure tlas;

// Incoming to vertex shader
struct VertexInfo
{
    float4 position;
    float4 color;
}

struct PushConstants
{
    uint vertex_buffer_idx;
}

[[vk::push_constant]]
ConstantBuffer<PushConstants> push_constants;

// Vertex shader to pixel shader
struct v2p
{
    float3 color : COLOR;
    float4 sv_position : SV_Position; // Specify the position of a vertex in screen space (after projection).
};

// Vertex shader
[shader("vertex")]
v2p vertexMain(uint vertexID: SV_VertexID)
{
    ByteAddressBuffer vertex_buffer = storage_buffers[push_constants.vertex_buffer_idx];
    VertexInfo info = vertex_buffer.Load<VertexInfo>(vertexID * sizeof(VertexInfo));

    v2p output;
    output.color = info.color.rgb;
    output.sv_position = info.position;

    return output;
}

// Pixel shader
[shader("pixel")]
float4 fragmentMain(v2p input)
    : SV_Target
{
    return float4(input.color, 1.0);
}
