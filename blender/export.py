import bpy
import struct

def write_sinister_header(file):
    file.write(b'**************************************************************\r\n')
    file.write(b'** Sinister Model File - Copyright(C) 1999 Sinister Games, Inc\r\n')
    file.write(b'** ASEParser Creation date: 12:18:34, Apr  1 1999\r\n')
    file.write(b'**\r\n')
    file.write(b'** SMF Version: SMF V1.1\r\n')
    file.write(b'** Model Name: AgStHs-MetalShack\r\n')
    file.write(b'** Created: 1:0:55, 4/20/1999\r\n')
    file.write(b'**************************************************************\r\n')


def write_fixed_string(file, text, size):
    # write the text as binary data to the file.
    file.write(text)
    file.write(b'\00' * (size - len(text)))


class Vertex:
    def __init__(self, i, v):
        self.index = i
        self.position = v.co
        self.normal = [0, 0, 0]
        self.uv = [0, 0]

    def write(self, file):
        file.write(struct.pack('I', self.index))
        file.write(struct.pack('fff', *self.position))

        file.write(struct.pack('II', 0, 0)) # unknown, unknown

        file.write(struct.pack('ff', *self.uv))
        file.write(struct.pack('fff', *self.normal))


class Face:
    def __init__(self, i, polygon):
        assert(len(polygon.vertices) <= 3)
        self.index = i
        self.indices = polygon.vertices[:3]

    def write(self, file):
        file.write(struct.pack('I', self.index))
        file.write(struct.pack('III', *self.indices))


class Mesh:
    def __init__(self, mesh):
        self.name = mesh.name
        self.texture = 'oil_helipad.bmp'
        self.vertices = [Vertex(i, v) for i, v in enumerate(mesh.vertices)]
        self.faces = [Face(i, p) for i, p in enumerate(mesh.polygons)]


    def write(self, file):
        write_fixed_string(file, self.name.encode(encoding='utf-8'), 128)
        write_fixed_string(file, self.texture.encode(encoding='utf-8'), 128)
        file.write(struct.pack('II', len(self.vertices), len(self.faces)))

        for vertex in self.vertices:
            vertex.write(file)

        for face in self.faces:
            face.write(file)


class Node:
    def __init__(self, node):
        if node.parent:
            parent_name = node.parent.name
        else:
            parent_name = '<root>'

        self.name = node.name
        self.parent_name = parent_name
        self.tree_id = 0
        self.position = node.location
        self.rotation = node.rotation_quaternion
        self.meshes = [Mesh(node.data)]
        self.bounding_boxes = []

    def write(self, file):
        write_fixed_string(file, self.name.encode(encoding='utf-8'), 128)
        write_fixed_string(file, self.parent_name.encode(encoding='utf-8'), 128)

        file.write(struct.pack('I', self.tree_id))
        file.write(struct.pack('fff', *self.position))
        # Quaternions has the W first in blender.
        file.write(struct.pack('ffff',  self.rotation[1], self.rotation[2], self.rotation[3], self.rotation[0]))
        file.write(struct.pack('II', len(self.meshes), len(self.bounding_boxes)))
        # unknown if version > 1
        file.write(struct.pack('I', 0))

        for mesh in self.meshes:
            mesh.write(file)

        for bounding_box in self.bounding_boxes:
            bounding_box.write(file)


class Model:
    def __init__(self, name):
        self.name = name
        self.unknown = [100, 1, 1]
        self.nodes = []

    def add_node(self, node):
        new_node = Node(node)
        print('Adding node', node.name)
        self.nodes.append(new_node)

        for child in node.children:
            self.add_node(child)

    def write(self, file):
        write_fixed_string(file, self.name.encode(encoding='utf-8'), 128)
        file.write(struct.pack('fff', *self.unknown))
        file.write(struct.pack('ff', 1.0, 1.0))
        file.write(struct.pack('I', len(self.nodes)))
        for node in self.nodes:
            node.write(file)


def build_node(nodes_out, node):
    node = {}
    node['name'] = type(node)
    nodes_out.append(node)

    '''
    write_fixed_string(file, node.name.encode(encoding='utf-8'), 128)
    file.write(b'\r\n')

    # get the node's parent name
    if node.parent:
        parent_name = node.parent.name
    else:
        parent_name = '<root>'
    write_fixed_string(file, parent_name.encode(encoding='utf-8'), 128)
    file.write(b'\r\n')

    for child in node.children:
        write_node(file, child)

    return

    # Write the tree_id
    file.write(struct.pack('I', 0))

    # position
    file.write(struct.pack('fff', *node.location))

    # rotation (in quaternion)
    file.write(struct.pack('ffff', *node.rotation_quaternion))

    # mesh_count: u32
    # bounding_box_count: u32
    file.write(struct.pack('II', len(node.data.polygons), 0))

    # Some extra myserious thing if version > 1
    file.write(struct.pack('I', 0))

    for mesh in node.data.polygons:
        write_fixed_string(file, node.data.name.encode(encoding='utf-8'), 128)
        write_fixed_string(file, b'', 128)

        # Write the number of vertices in the polygon
        file.write(struct.pack('I', len(mesh.vertices)))

        # Write the vertex indices
        for vertex_index in mesh.vertices:
            file.write(struct.pack('I', vertex_index))

        # Write the number of UV layers
        uv_layers = node.data.uv_layers
        file.write(struct.pack('I', len(uv_layers)))

        # Write the UV coordinates for each layer
        for uv_layer in uv_layers:
            for loop_index in mesh.loop_indices:
                uv = uv_layer.data[loop_index].uv
                file.write(struct.pack('ff', uv.x, uv.y))
        '''



# Function to export node data to a binary file
def export_selected_node_to_binary(filepath):
    selected_node = bpy.context.view_layer.objects.active

    if selected_node is None or selected_node.type != 'MESH':
        print("No active mesh selected.")
        return

    model = Model(selected_node.name)
    model.add_node(selected_node)

    print(model.__dict__)

    # Open the binary file for writing
    with open(filepath, 'wb') as file:
        write_sinister_header(file)
        # Write some weird unknown bytes: 1A FA 31 C1 | DE ED 42 13
        file.write(b'\x1A\xFA\x31\xC1\xDE\xED\x42\x13')

        # Write the version string.
        write_fixed_string(file, b'SMF V1.1', 16)

        model.write(file)

        # # Write the name of the model.
        # write_fixed_string(file, selected_node.name.encode(encoding='utf-8'), 128)

        # # Write the mysterious scale something.
        # file.write(struct.pack('fff', 100, 1, 1))

        # # Write the node count as a u32.
        # node_count = len(nodes)
        # file.write(struct.pack('I', node_count))

        # for node in nodes:
        #     write_fixed_string(file, node.name.encode(encoding='utf-8'), 128)
        #     file.write(b'\r\n')

        #     # get the node's parent name
        #     if node.parent:
        #         parent_name = node.parent.name
        #     else:
        #         parent_name = '<root>'
        #     write_fixed_string(file, parent_name.encode(encoding='utf-8'), 128)
        #     file.write(b'\r\n')



    print(f"Node '{selected_node.name}' exported to {filepath}")

# Specify the path to save the binary file
export_path = bpy.path.abspath('C:\\games\\shadow_company\\data-wip\\models\\test.smf')

# Call the function to export the selected node
export_selected_node_to_binary(export_path)
