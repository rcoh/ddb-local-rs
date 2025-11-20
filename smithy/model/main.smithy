$version: "2"

namespace com.amazonaws.dynamodb

use aws.api#service
use aws.protocols#awsJson1_0
use smithy.framework#ValidationException

string ArchivalReason

structure ArchivalSummary {
    ArchivalDateTime: Date
    ArchivalReason: ArchivalReason
    ArchivalBackupArn: BackupArn
}

structure AttributeDefinition {
    @required
    AttributeName: KeySchemaAttributeName

    @required
    AttributeType: ScalarAttributeType
}

list AttributeDefinitions {
    member: AttributeDefinition
}

map AttributeMap {
    key: AttributeName
    value: AttributeValue
}

string AttributeName

list AttributeNameList {
    member: AttributeName
}

union AttributeValue {
    S: StringAttributeValue
    N: NumberAttributeValue
    B: BinaryAttributeValue
    SS: StringSetAttributeValue
    NS: NumberSetAttributeValue
    BS: BinarySetAttributeValue
    M: MapAttributeValue
    L: ListAttributeValue
    NULL: NullAttributeValue
    BOOL: BooleanAttributeValue
}

list AttributeValueList {
    member: AttributeValue
}

boolean Backfilling

string BackupArn

enum BillingMode {
    PROVISIONED = "PROVISIONED"
    PAY_PER_REQUEST = "PAY_PER_REQUEST"
}

structure BillingModeSummary {
    BillingMode: BillingMode
    LastUpdateToPayPerRequestDateTime: Date
}

blob BinaryAttributeValue

list BinarySetAttributeValue {
    member: BinaryAttributeValue
}

boolean BooleanAttributeValue

boolean BooleanObject

structure Capacity {
    ReadCapacityUnits: ConsumedCapacityUnits
    WriteCapacityUnits: ConsumedCapacityUnits
    CapacityUnits: ConsumedCapacityUnits
}

enum ComparisonOperator {
    EQ = "EQ"
    NE = "NE"
    IN = "IN"
    LE = "LE"
    LT = "LT"
    GE = "GE"
    GT = "GT"
    BETWEEN = "BETWEEN"
    NOT_NULL = "NOT_NULL"
    NULL = "NULL"
    CONTAINS = "CONTAINS"
    NOT_CONTAINS = "NOT_CONTAINS"
    BEGINS_WITH = "BEGINS_WITH"
}

string ConditionExpression

@error("client")
structure ConditionalCheckFailedException {
    message: ErrorMessage
}

enum ConditionalOperator {
    AND = "AND"
    OR = "OR"
}

boolean ConsistentRead

structure ConsumedCapacity {
    TableName: TableName
    CapacityUnits: ConsumedCapacityUnits
    ReadCapacityUnits: ConsumedCapacityUnits
    WriteCapacityUnits: ConsumedCapacityUnits
    Table: Capacity
    LocalSecondaryIndexes: SecondaryIndexesCapacityMap
    GlobalSecondaryIndexes: SecondaryIndexesCapacityMap
}

double ConsumedCapacityUnits

operation CreateTable {
    input: CreateTableInput
    output: CreateTableOutput
    errors: [
        ValidationException
        InternalServerError
        InvalidEndpointException
        LimitExceededException
        ResourceInUseException
    ]
}

structure CreateTableInput {
    @required
    AttributeDefinitions: AttributeDefinitions

    @required
    TableName: TableName

    @required
    KeySchema: KeySchema

    LocalSecondaryIndexes: LocalSecondaryIndexList

    GlobalSecondaryIndexes: GlobalSecondaryIndexList

    BillingMode: BillingMode

    ProvisionedThroughput: ProvisionedThroughput

    StreamSpecification: StreamSpecification

    SSESpecification: SSESpecification

    Tags: TagList
}

structure CreateTableOutput {
    TableDescription: TableDescription
}

timestamp Date

@awsJson1_0
@service(sdkId: "DynamoDB")
service DynamoDB_20120810 {
    version: "2012-08-10"
    operations: [
        GetItem
        PutItem
        CreateTable
    ]
}

string ErrorMessage

map ExpectedAttributeMap {
    key: AttributeName
    value: ExpectedAttributeValue
}

structure ExpectedAttributeValue {
    Value: AttributeValue
    Exists: BooleanObject
    ComparisonOperator: ComparisonOperator
    AttributeValueList: AttributeValueList
}

map ExpressionAttributeNameMap {
    key: ExpressionAttributeNameVariable
    value: AttributeName
}

string ExpressionAttributeNameVariable

map ExpressionAttributeValueMap {
    key: ExpressionAttributeValueVariable
    value: AttributeValue
}

string ExpressionAttributeValueVariable

operation GetItem {
    input: GetItemInput
    output: GetItemOutput
    errors: [
        ValidationException
        InternalServerError
        InvalidEndpointException
        ProvisionedThroughputExceededException
        RequestLimitExceeded
        ResourceNotFoundException
    ]
}

structure GetItemInput {
    @required
    TableName: TableName

    @required
    Key: Key

    AttributesToGet: AttributeNameList

    ConsistentRead: ConsistentRead

    ReturnConsumedCapacity: ReturnConsumedCapacity

    ProjectionExpression: ProjectionExpression

    ExpressionAttributeNames: ExpressionAttributeNameMap
}

structure GetItemOutput {
    Item: AttributeMap
    ConsumedCapacity: ConsumedCapacity
}

structure GlobalSecondaryIndex {
    @required
    IndexName: IndexName

    @required
    KeySchema: KeySchema

    @required
    Projection: Projection

    ProvisionedThroughput: ProvisionedThroughput
}

structure GlobalSecondaryIndexDescription {
    IndexName: IndexName
    KeySchema: KeySchema
    Projection: Projection
    IndexStatus: IndexStatus
    Backfilling: Backfilling
    ProvisionedThroughput: ProvisionedThroughputDescription
    IndexSizeBytes: Long
    ItemCount: Long
    IndexArn: String
}

list GlobalSecondaryIndexDescriptionList {
    member: GlobalSecondaryIndexDescription
}

list GlobalSecondaryIndexList {
    member: GlobalSecondaryIndex
}

string IndexName

enum IndexStatus {
    CREATING = "CREATING"
    UPDATING = "UPDATING"
    DELETING = "DELETING"
    ACTIVE = "ACTIVE"
}

@error("server")
structure InternalServerError {
    message: ErrorMessage
}

@error("client")
structure InvalidEndpointException {
    Message: String
}

map ItemCollectionKeyAttributeMap {
    key: AttributeName
    value: AttributeValue
}

structure ItemCollectionMetrics {
    ItemCollectionKey: ItemCollectionKeyAttributeMap
    SizeEstimateRangeGB: ItemCollectionSizeEstimateRange
}

double ItemCollectionSizeEstimateBound

list ItemCollectionSizeEstimateRange {
    member: ItemCollectionSizeEstimateBound
}

@error("client")
structure ItemCollectionSizeLimitExceededException {
    message: ErrorMessage
}

string KMSMasterKeyArn

string KMSMasterKeyId

map Key {
    key: AttributeName
    value: AttributeValue
}

list KeySchema {
    member: KeySchemaElement
}

string KeySchemaAttributeName

structure KeySchemaElement {
    @required
    AttributeName: KeySchemaAttributeName

    @required
    KeyType: KeyType
}

enum KeyType {
    HASH = "HASH"
    RANGE = "RANGE"
}

@error("client")
structure LimitExceededException {
    message: ErrorMessage
}

list ListAttributeValue {
    member: AttributeValue
}

structure LocalSecondaryIndex {
    @required
    IndexName: IndexName

    @required
    KeySchema: KeySchema

    @required
    Projection: Projection
}

structure LocalSecondaryIndexDescription {
    IndexName: IndexName
    KeySchema: KeySchema
    Projection: Projection
    IndexSizeBytes: Long
    ItemCount: Long
    IndexArn: String
}

list LocalSecondaryIndexDescriptionList {
    member: LocalSecondaryIndexDescription
}

list LocalSecondaryIndexList {
    member: LocalSecondaryIndex
}

long Long

map MapAttributeValue {
    key: AttributeName
    value: AttributeValue
}

string NonKeyAttributeName

list NonKeyAttributeNameList {
    member: NonKeyAttributeName
}

long NonNegativeLongObject

boolean NullAttributeValue

string NumberAttributeValue

list NumberSetAttributeValue {
    member: NumberAttributeValue
}

long PositiveLongObject

structure Projection {
    ProjectionType: ProjectionType
    NonKeyAttributes: NonKeyAttributeNameList
}

string ProjectionExpression

enum ProjectionType {
    ALL = "ALL"
    KEYS_ONLY = "KEYS_ONLY"
    INCLUDE = "INCLUDE"
}

structure ProvisionedThroughput {
    @required
    ReadCapacityUnits: PositiveLongObject

    @required
    WriteCapacityUnits: PositiveLongObject
}

structure ProvisionedThroughputDescription {
    LastIncreaseDateTime: Date
    LastDecreaseDateTime: Date
    NumberOfDecreasesToday: PositiveLongObject
    ReadCapacityUnits: NonNegativeLongObject
    WriteCapacityUnits: NonNegativeLongObject
}

@error("client")
structure ProvisionedThroughputExceededException {
    message: ErrorMessage
}

structure ProvisionedThroughputOverride {
    ReadCapacityUnits: PositiveLongObject
}

operation PutItem {
    input: PutItemInput
    output: PutItemOutput
    errors: [
        ValidationException
        ConditionalCheckFailedException
        InternalServerError
        InvalidEndpointException
        ItemCollectionSizeLimitExceededException
        ProvisionedThroughputExceededException
        RequestLimitExceeded
        ResourceNotFoundException
        TransactionConflictException
    ]
}

structure PutItemInput {
    @required
    TableName: TableName

    @required
    Item: PutItemInputAttributeMap

    Expected: ExpectedAttributeMap

    ReturnValues: ReturnValue

    ReturnConsumedCapacity: ReturnConsumedCapacity

    ReturnItemCollectionMetrics: ReturnItemCollectionMetrics

    ConditionalOperator: ConditionalOperator

    ConditionExpression: ConditionExpression

    ExpressionAttributeNames: ExpressionAttributeNameMap

    ExpressionAttributeValues: ExpressionAttributeValueMap
}

map PutItemInputAttributeMap {
    key: AttributeName
    value: AttributeValue
}

structure PutItemOutput {
    Attributes: AttributeMap
    ConsumedCapacity: ConsumedCapacity
    ItemCollectionMetrics: ItemCollectionMetrics
}

string RegionName

structure ReplicaDescription {
    RegionName: RegionName
    ReplicaStatus: ReplicaStatus
    ReplicaStatusDescription: ReplicaStatusDescription
    ReplicaStatusPercentProgress: ReplicaStatusPercentProgress
    KMSMasterKeyId: KMSMasterKeyId
    ProvisionedThroughputOverride: ProvisionedThroughputOverride
    GlobalSecondaryIndexes: ReplicaGlobalSecondaryIndexDescriptionList
    ReplicaInaccessibleDateTime: Date
}

list ReplicaDescriptionList {
    member: ReplicaDescription
}

structure ReplicaGlobalSecondaryIndexDescription {
    IndexName: IndexName
    ProvisionedThroughputOverride: ProvisionedThroughputOverride
}

list ReplicaGlobalSecondaryIndexDescriptionList {
    member: ReplicaGlobalSecondaryIndexDescription
}

enum ReplicaStatus {
    CREATING = "CREATING"
    CREATION_FAILED = "CREATION_FAILED"
    UPDATING = "UPDATING"
    DELETING = "DELETING"
    ACTIVE = "ACTIVE"
    REGION_DISABLED = "REGION_DISABLED"
    INACCESSIBLE_ENCRYPTION_CREDENTIALS = "INACCESSIBLE_ENCRYPTION_CREDENTIALS"
}

string ReplicaStatusDescription

string ReplicaStatusPercentProgress

@error("client")
structure RequestLimitExceeded {
    message: ErrorMessage
}

@error("client")
structure ResourceInUseException {
    message: ErrorMessage
}

@error("client")
structure ResourceNotFoundException {
    message: ErrorMessage
}

boolean RestoreInProgress

structure RestoreSummary {
    SourceBackupArn: BackupArn

    SourceTableArn: TableArn

    @required
    RestoreDateTime: Date

    @required
    RestoreInProgress: RestoreInProgress
}

enum ReturnConsumedCapacity {
    INDEXES = "INDEXES"
    TOTAL = "TOTAL"
    NONE = "NONE"
}

enum ReturnItemCollectionMetrics {
    SIZE = "SIZE"
    NONE = "NONE"
}

enum ReturnValue {
    NONE = "NONE"
    ALL_OLD = "ALL_OLD"
    UPDATED_OLD = "UPDATED_OLD"
    ALL_NEW = "ALL_NEW"
    UPDATED_NEW = "UPDATED_NEW"
}

structure SSEDescription {
    Status: SSEStatus
    SSEType: SSEType
    KMSMasterKeyArn: KMSMasterKeyArn
    InaccessibleEncryptionDateTime: Date
}

boolean SSEEnabled

structure SSESpecification {
    Enabled: SSEEnabled
    SSEType: SSEType
    KMSMasterKeyId: KMSMasterKeyId
}

enum SSEStatus {
    ENABLING = "ENABLING"
    ENABLED = "ENABLED"
    DISABLING = "DISABLING"
    DISABLED = "DISABLED"
    UPDATING = "UPDATING"
}

enum SSEType {
    AES256 = "AES256"
    KMS = "KMS"
}

enum ScalarAttributeType {
    S = "S"
    N = "N"
    B = "B"
}

map SecondaryIndexesCapacityMap {
    key: IndexName
    value: Capacity
}

string StreamArn

boolean StreamEnabled

structure StreamSpecification {
    @required
    StreamEnabled: StreamEnabled

    StreamViewType: StreamViewType
}

enum StreamViewType {
    NEW_IMAGE = "NEW_IMAGE"
    OLD_IMAGE = "OLD_IMAGE"
    NEW_AND_OLD_IMAGES = "NEW_AND_OLD_IMAGES"
    KEYS_ONLY = "KEYS_ONLY"
}

string String

string StringAttributeValue

list StringSetAttributeValue {
    member: StringAttributeValue
}

string TableArn

structure TableDescription {
    AttributeDefinitions: AttributeDefinitions
    TableName: TableName
    KeySchema: KeySchema
    TableStatus: TableStatus
    CreationDateTime: Date
    ProvisionedThroughput: ProvisionedThroughputDescription
    TableSizeBytes: Long
    ItemCount: Long
    TableArn: String
    TableId: TableId
    BillingModeSummary: BillingModeSummary
    LocalSecondaryIndexes: LocalSecondaryIndexDescriptionList
    GlobalSecondaryIndexes: GlobalSecondaryIndexDescriptionList
    StreamSpecification: StreamSpecification
    LatestStreamLabel: String
    LatestStreamArn: StreamArn
    GlobalTableVersion: String
    Replicas: ReplicaDescriptionList
    RestoreSummary: RestoreSummary
    SSEDescription: SSEDescription
    ArchivalSummary: ArchivalSummary
}

string TableId

string TableName

enum TableStatus {
    CREATING = "CREATING"
    UPDATING = "UPDATING"
    DELETING = "DELETING"
    ACTIVE = "ACTIVE"
    INACCESSIBLE_ENCRYPTION_CREDENTIALS = "INACCESSIBLE_ENCRYPTION_CREDENTIALS"
    ARCHIVING = "ARCHIVING"
    ARCHIVED = "ARCHIVED"
}

structure Tag {
    @required
    Key: TagKeyString

    @required
    Value: TagValueString
}

string TagKeyString

list TagList {
    member: Tag
}

string TagValueString

@error("client")
structure TransactionConflictException {
    message: ErrorMessage
}
