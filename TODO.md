## Check if the object exists in s3 before uploading again

## Attach a file validity header to the put request
    - https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html

## Use multipart uploads
    - https://docs.aws.amazon.com/AmazonS3/latest/userguide/mpuoverview.html
    - https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/client/struct.Client.html#method.create_multipart_upload
    - https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/client/struct.Client.html#method.upload_part

## Build up the additions changes set from a list of commits 

## For status + add need better file comparison

Right now files are only either in or out of the index, we need to know if they
change as well.

In general it would be nice to have status build up a data object and then
print that instead of just kind of writing shit out to sdout


## Get branching working

I already have refs working, at this point, it's just a matter of adding the
various branch commands.

## Print the current branch name in status

Just do it

## Remote Fetch

I have the remote db being fetched down and I can use that to diff a subsequent
push, but there isn't the fetch down of the remote data to sync up.
