#!/usr/bin/env python3
import json

with open('model/dynamo.json.bak') as f:
    model = json.load(f)

shapes = model['shapes']

# Find dependencies recursively
def find_deps(shape_id, visited=None):
    if visited is None:
        visited = set()
    if shape_id in visited or shape_id not in shapes:
        return visited
    visited.add(shape_id)
    
    shape = shapes[shape_id]
    
    # Check various shape properties for references
    if 'input' in shape:
        find_deps(shape['input']['target'], visited)
    if 'output' in shape:
        find_deps(shape['output']['target'], visited)
    if 'errors' in shape:
        for err in shape['errors']:
            find_deps(err['target'], visited)
    if 'members' in shape:
        for member in shape['members'].values():
            find_deps(member['target'], visited)
    if 'member' in shape:  # for lists
        find_deps(shape['member']['target'], visited)
    if 'key' in shape:  # for maps
        find_deps(shape['key']['target'], visited)
    if 'value' in shape:  # for maps
        find_deps(shape['value']['target'], visited)
    
    return visited

# Start with GetItem, PutItem, and CreateTable
needed = set()
find_deps('com.amazonaws.dynamodb#GetItem', needed)
find_deps('com.amazonaws.dynamodb#PutItem', needed)
find_deps('com.amazonaws.dynamodb#CreateTable', needed)

# Also need the service shape
needed.add('com.amazonaws.dynamodb#DynamoDB_20120810')

# Convert to Smithy IDL
def to_smithy_type(shape_type):
    type_map = {
        'string': 'String',
        'integer': 'Integer',
        'long': 'Long',
        'boolean': 'Boolean',
        'blob': 'Blob',
        'double': 'Double',
        'timestamp': 'Timestamp'
    }
    return type_map.get(shape_type, shape_type)

def format_doc(doc_str):
    if not doc_str:
        return ""
    # Simple formatting - just add /// prefix
    lines = doc_str.split('\n')
    return '\n'.join(f'/// {line}' for line in lines if line.strip())

output = ['$version: "2"', '', 'namespace com.amazonaws.dynamodb', '']

# Add necessary imports
output.append('use aws.protocols#awsJson1_0')
output.append('use aws.api#service')
output.append('use smithy.framework#ValidationException')
output.append('')

# Process shapes
for shape_id in sorted(needed):
    if shape_id not in shapes:
        continue
    
    shape = shapes[shape_id]
    shape_name = shape_id.split('#')[1]
    shape_type = shape['type']
    
    if shape_type == 'service':
        output.append('@awsJson1_0')
        output.append(f'@service(sdkId: "DynamoDB")')
        output.append(f'service {shape_name} {{')
        output.append(f'    version: "{shape["version"]}"')
        output.append('    operations: [')
        output.append('        GetItem')
        output.append('        PutItem')
        output.append('        CreateTable')
        output.append('    ]')
        output.append('}')
        output.append('')
        
    elif shape_type == 'operation':
        output.append(f'operation {shape_name} {{')
        if 'input' in shape:
            input_name = shape['input']['target'].split('#')[1]
            output.append(f'    input: {input_name}')
        if 'output' in shape:
            output_name = shape['output']['target'].split('#')[1]
            output.append(f'    output: {output_name}')
        if 'errors' in shape:
            errors = [e['target'].split('#')[1] for e in shape['errors']]
            # Add ValidationException if not already present
            if 'ValidationException' not in errors:
                errors.insert(0, 'ValidationException')
            output.append(f'    errors: [')
            for error in errors:
                output.append(f'        {error}')
            output.append(f'    ]')
        output.append('}')
        output.append('')
        
    elif shape_type == 'structure':
        # Check if this is an error structure
        is_error = 'traits' in shape and 'smithy.api#error' in shape['traits']
        if is_error:
            error_type = shape['traits']['smithy.api#error']
            output.append(f'@error("{error_type}")')
        output.append(f'structure {shape_name} {{')
        if 'members' in shape:
            for member_name, member_info in shape['members'].items():
                target = member_info['target'].split('#')[1]
                required = '@required ' if 'traits' in member_info and 'smithy.api#required' in member_info['traits'] else ''
                output.append(f'    {required}{member_name}: {target}')
        output.append('}')
        output.append('')
    
    elif shape_type == 'union':
        output.append(f'union {shape_name} {{')
        if 'members' in shape:
            for member_name, member_info in shape['members'].items():
                target = member_info['target'].split('#')[1]
                output.append(f'    {member_name}: {target}')
        output.append('}')
        output.append('')
        
    elif shape_type == 'map':
        key_target = shape['key']['target'].split('#')[1]
        value_target = shape['value']['target'].split('#')[1]
        output.append(f'map {shape_name} {{')
        output.append(f'    key: {key_target}')
        output.append(f'    value: {value_target}')
        output.append('}')
        output.append('')
        
    elif shape_type == 'list':
        member_target = shape['member']['target'].split('#')[1]
        output.append(f'list {shape_name} {{')
        output.append(f'    member: {member_target}')
        output.append('}')
        output.append('')
        
    elif shape_type == 'string':
        if 'traits' in shape and 'smithy.api#enum' in shape['traits']:
            output.append(f'enum {shape_name} {{')
            for enum_val in shape['traits']['smithy.api#enum']:
                output.append(f'    {enum_val["name"]} = "{enum_val["value"]}"')
            output.append('}')
            output.append('')
        else:
            output.append(f'string {shape_name}')
            output.append('')
    
    elif shape_type in ['integer', 'long', 'boolean', 'blob', 'double', 'timestamp']:
        output.append(f'{shape_type} {shape_name}')
        output.append('')

print('\n'.join(output))
