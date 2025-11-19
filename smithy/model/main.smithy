$version: "2"

namespace com.amazonaws.dynamodb

use aws.api#service
use aws.protocols#awsJson1_0
use smithy.framework#ValidationException

@awsJson1_0
@service(sdkId: "DynamoDB")
service DynamoDB_20120810 {
    version: "2012-08-10"
    operations: [
        GetItem
        PutItem
    ]
}

operation GetItem {
    input: GetItemInput
    output: GetItemOutput
    errors: [
        ValidationException
        InternalServerError
        ProvisionedThroughputExceededException
        ResourceNotFoundException
    ]
}

structure GetItemInput {
    @required
    TableName: String

    @required
    Key: Key

    ConsistentRead: Boolean

    ProjectionExpression: String

    ExpressionAttributeNames: ExpressionAttributeNameMap
}

structure GetItemOutput {
    Item: AttributeMap
}

operation PutItem {
    input: PutItemInput
    output: PutItemOutput
    errors: [
        ValidationException
        InternalServerError
        ProvisionedThroughputExceededException
        ResourceNotFoundException
        ConditionalCheckFailedException
    ]
}

structure PutItemInput {
    @required
    TableName: String

    @required
    Item: AttributeMap

    ConditionExpression: String

    ExpressionAttributeNames: ExpressionAttributeNameMap

    ExpressionAttributeValues: ExpressionAttributeValueMap
}

structure PutItemOutput {
    Attributes: AttributeMap
}

map Key {
    key: String
    value: AttributeValue
}

map AttributeMap {
    key: String
    value: AttributeValue
}

map ExpressionAttributeNameMap {
    key: String
    value: String
}

map ExpressionAttributeValueMap {
    key: String
    value: AttributeValue
}

union AttributeValue {
    S: String
    N: String
    B: Blob
    SS: StringSet
    NS: NumberSet
    BS: BinarySet
    M: AttributeMap
    L: AttributeList
    NULL: Boolean
    BOOL: Boolean
}

list StringSet {
    member: String
}

list NumberSet {
    member: String
}

list BinarySet {
    member: Blob
}

list AttributeList {
    member: AttributeValue
}

@error("server")
structure InternalServerError {
    message: String
}

@error("client")
structure ProvisionedThroughputExceededException {
    message: String
}

@error("client")
structure ResourceNotFoundException {
    message: String
}

@error("client")
structure ConditionalCheckFailedException {
    message: String
    Item: AttributeMap
}
