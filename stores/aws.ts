import {create, StateCreator} from 'zustand';
import {devtools, persist} from 'zustand/middleware';

export interface S3Bucket{
  name: string;
  creation_date: string;
  bucket_arn: string;
  bucket_region: string;
}

export interface AwsState {
  profile?: string;
  setProfile: (profile: string) => void;
  profiles: string[];
  setProfiles: (profiles: string[]) => void;
  s3Buckets: S3Bucket[];
  setS3Buckets: (buckets: S3Bucket[]) => void;
}

const initializer: StateCreator<AwsState> = (set) => ({
  profile: undefined,
  setProfile: (profile: string) => set({ profile }),
  profiles: [],
  setProfiles: (profiles: string[]) => set({ profiles }),
  s3Buckets: [],
  setS3Buckets: (buckets: S3Bucket[]) => set({ s3Buckets: buckets }),
});

function middleware(initializer: StateCreator<AwsState>) {
  return devtools(
    persist(initializer, {
      name: "aws-state",
    })
  );
}

export const useAwsStore = create<AwsState>()(middleware(initializer));
