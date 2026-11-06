#pragma once

#include <cstdint>
#include <memory>
#include <vector>

namespace audiobridge {

class OboeAudioSinkCallback {
public:
    virtual ~OboeAudioSinkCallback() = default;
    virtual void onUnderrun(int64_t count) = 0;
};

class OboeAudioSink {
public:
    OboeAudioSink();
    ~OboeAudioSink();

    bool open(int32_t sampleRate, int32_t channelCount);
    bool start();
    void pause();
    void stop();
    void close();

    int32_t write(const int16_t* audioData, int32_t numFrames, int64_t timeoutNanoseconds);
    int32_t getFramesPerBurst() const;
    int32_t getBufferSizeInFrames() const;
    int64_t getUnderrunCount() const;

    void setCallback(OboeAudioSinkCallback* callback);

private:
    class Impl;
    std::unique_ptr<Impl> mImpl;
};

} // namespace audiobridge
