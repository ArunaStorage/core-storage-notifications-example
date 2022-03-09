# Subscribe to data changes in the NFDI4BioDiv Core-Storage
This small example demonstrates how to receive notifications about updates, insertions and deletions
of elements within the [NFDI4BioDiv Core-Storage](https://kb.gfbio.org/display/NFDI/CORE-Storage).
This tutorial assumes you are familiar with the basics of the NFDI4BioDiv Core-Storage. If not
consider looking at [this](https://github.com/koerberm/core-storage-example) tutorial first.

## Basics
Notifications basically work like any other pub/sub system (in fact, the mechanism is backed by [nats.io](https://nats.io)).
Assuming you have the permissions, you can use the API to subscribe to changes of any element in the storage hierarchy.
More precisely, you can listen to changes on:
- Projects
- Datasets
- DatasetVersions
- ObjectGroups

In addition, it allows you to subscribe to a higher level resource (e.g., a dataset) and also receive notifications for 
all sub resrouces (e.g., DatasetVersion, ObjectGroups).


## Subscriptions
Subscriptions are organized in so-called EventStreamingGroups. In order to subscribe to certain notification you
first create such a group by specifying the resources you want to receive updates for. After registering your group
with the backend, you receive a unique group id, that is used request a notification stream.
Groups are persistent and allow multiple consumers to cooperatively consume the notifications of the group.
This means, that the backend takes care, that a notification is sent to only a single consumer process of the 
corresponding group.

### Notifications
Once subscribed to a group, the client receives a stream of notification batches. Those batches consist of a set of
notifications and an acknowledgement id. This id is used to signal the server that all messages of the batch
were processed successfully. This means that no other client process will receive those messages.

In turn, if a batch is not acknowledged before requesting the next batch, the server will resend those messages to any
member of the group.

## API
In the source folder you find an example application written in the [rust](https://www.rust-lang.org/) programming
language. It uses the stubs provided [here](https://github.com/ScienceObjectsDB/rust-api). However, there are
implementations for other programming languages available (a full list is available [here](https://github.com/ScienceObjectsDB/Documentation#implementations).

The example listens to changes in all components of a given project. In order to
run it, you must insert a valid *project id* and a valid *API token*.

